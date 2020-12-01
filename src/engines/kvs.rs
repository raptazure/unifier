use crate::error::{KvsError, Result};
use crate::KvsEngine;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

// ========================= KvStore =========================
const COMPACTION_THRESHOLD: u64 = 4 * 1024 * 1024;

/// Used to store a string key to a string value.
///
/// # Example
///
/// ```
/// # use kvs::KvStore;
/// # use kvs::KvsEngine;
/// # use std::env::current_dir;
/// let mut kvs = KvStore::open(current_dir().unwrap()).unwrap();
///
/// kvs.set("key".to_string(), "value".to_string());
///
/// let val = kvs.get("key".to_string()).unwrap();
/// assert_eq!(val, Some("value".to_string()));
///
/// kvs.remove("key".to_string());
/// let val = kvs.get("key".to_string()).unwrap();
/// assert_eq!(val, None);
/// ```
pub struct KvStore {
    path: Arc<PathBuf>,
    writer: Arc<Mutex<KvStoreWriter>>,
    reader: KvStoreReader,
    index: Arc<RwLock<HashMap<String, CommandOffset>>>,
}

impl KvStore {
    /// Open the KvStore at a given path.
    /// Return the KvStore.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        let path = path.join("kvs.db");
        fs::create_dir_all(&path)?;

        let path = Arc::new(path);
        let index = Arc::new(RwLock::new(HashMap::new()));
        let reader = KvStoreReader::new(Arc::clone(&path), Arc::clone(&index));

        let gens = generations(&path)?;
        for gen in gens.iter() {
            let path = db_path(&path, *gen);
            let mut new_reader = BufReader::new(File::open(path)?);

            load_index(*gen, &mut new_reader, &mut index.write().unwrap())?;
            reader.add_reader(gen, new_reader);
        }

        let current_gen = gens.last().unwrap_or(&0) + 1;
        let (new_writer, new_reader) = new_db_log(&db_path(&path, current_gen))?;
        reader.add_reader(&current_gen, new_reader);

        let writer = KvStoreWriter::new(
            Arc::clone(&path),
            new_writer,
            reader.clone(),
            Arc::clone(&index),
            current_gen,
        )?;
        let writer = Arc::new(Mutex::new(writer));

        Ok(KvStore {
            path: Arc::clone(&path),
            writer,
            reader,
            index,
        })
    }

    /// Compacting the Error file.
    /// To support concurrent, use generation to maintain the Error files.
    pub fn compact(&self) -> Result<()> {
        self.writer.lock().unwrap().compact()
    }
}

impl Clone for KvStore {
    fn clone(&self) -> Self {
        KvStore {
            path: Arc::clone(&self.path),
            writer: Arc::clone(&self.writer),
            reader: self.reader.clone(),
            index: Arc::clone(&self.index),
        }
    }
}

impl KvsEngine for KvStore {
    /// Sets the value of a string key to a string.
    /// Return an error if the value is not written successfully.
    ///
    /// # Example
    ///
    /// ```
    /// # use kvs::KvStore;
    /// # use kvs::KvsEngine;
    /// # use std::env::current_dir;
    /// let mut kvs = KvStore::open(current_dir().unwrap()).unwrap();
    /// kvs.set("key".to_string(), "value".to_string());
    /// ```
    fn set(&self, key: String, value: String) -> Result<()> {
        self.writer.lock().unwrap().set(key, value)
    }

    /// Gets the string value of the a string key.
    /// If the key does not exist, return None.
    /// Return an error if the value is not read successfully.
    ///
    /// # Example
    ///
    /// ```
    /// # use kvs::KvStore;
    /// # use kvs::KvsEngine;
    /// # use std::env::current_dir;
    /// let mut kvs = KvStore::open(current_dir().unwrap()).unwrap();
    /// let value = kvs.get("non-exist-key".to_string()).unwrap();
    ///
    /// assert_eq!(value, None);
    /// ```
    fn get(&self, key: String) -> Result<Option<String>> {
        if let Some(offset) = self.index.read().unwrap().get(&key) {
            let command = self.reader.read_command(offset)?;
            if let Command::Set { key: _, value } = command {
                Ok(Some(value))
            } else {
                unreachable!()
            }
        } else {
            Ok(None)
        }
    }

    /// Removes a given key.
    /// Return an error if the key does not exist or is not removed successfully.
    ///
    /// # Example
    ///
    /// ```
    /// # use kvs::KvStore;
    /// # use kvs::KvsEngine;
    /// # use std::env::current_dir;
    /// let mut kvs = KvStore::open(current_dir().unwrap()).unwrap();
    /// kvs.set("key".to_string(), "value".to_string());
    /// kvs.remove("key".to_string());
    ///
    /// let value = kvs.get("key".to_string()).unwrap();
    /// assert_eq!(value, None);
    /// ```
    fn remove(&self, key: String) -> Result<()> {
        self.writer.lock().unwrap().remove(key)
    }
}

// ========================= KvStoreReader =========================

/// A single thread key value reader.
///
/// Each thread own its reader for concurrently reading.
/// And `RefCell` provide inner mutability and `RwLock` for more reading operations than writing.
struct KvStoreReader {
    path: Arc<PathBuf>,
    readers: RefCell<HashMap<u64, BufReader<File>>>,
    index: Arc<RwLock<HashMap<String, CommandOffset>>>,
}

impl Clone for KvStoreReader {
    fn clone(&self) -> Self {
        KvStoreReader {
            path: Arc::clone(&self.path),
            readers: RefCell::new(HashMap::new()),
            index: Arc::clone(&self.index),
        }
    }
}

impl KvStoreReader {
    fn new(path: Arc<PathBuf>, index: Arc<RwLock<HashMap<String, CommandOffset>>>) -> Self {
        let readers = RefCell::new(HashMap::new());
        KvStoreReader {
            path: Arc::clone(&path),
            readers,
            index,
        }
    }

    fn read<F, R>(&self, gen: &u64, func: F) -> Result<R>
    where
        F: FnOnce(&mut BufReader<File>) -> Result<R> + Send,
    {
        let mut readers = self.readers.borrow_mut();

        if !readers.contains_key(gen) {
            let path = db_path(&self.path, *gen);
            let reader = BufReader::new(File::open(path)?);
            readers.insert(*gen, reader);
        }

        let reader = readers.get_mut(gen).unwrap();
        func(reader)
    }

    fn add_reader(&self, gen: &u64, reader: BufReader<File>) {
        self.readers.borrow_mut().insert(*gen, reader);
    }

    fn remove_reader(&self, gen: &u64) {
        self.readers.borrow_mut().remove(gen);
    }

    fn read_command(&self, offset: &CommandOffset) -> Result<Command> {
        let CommandOffset { gen, pos, len } = offset;
        self.read(gen, |reader| {
            reader.seek(SeekFrom::Start(*pos))?;

            let mut buffer = vec![0u8; *len as usize];
            reader.read_exact(&mut buffer)?;
            Ok(serde_json::from_slice(&buffer)?)
        })
    }
}

// ========================= KvStoreWriter =========================

struct PosBufWriter<T: Write + Seek> {
    writer: BufWriter<T>,
    pos: u64,
}

impl<T: Write + Seek> PosBufWriter<T> {
    fn new(mut writer: BufWriter<T>) -> Result<Self> {
        let pos = writer.seek(SeekFrom::End(0))?;
        Ok(PosBufWriter { writer, pos })
    }
}

impl<T: Write + Seek> Write for PosBufWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size = self.writer.write(buf)?;
        self.pos += size as u64;
        Ok(size)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<T: Write + Seek> Seek for PosBufWriter<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}

struct KvStoreWriter {
    path: Arc<PathBuf>,
    writer: PosBufWriter<File>,
    reader: KvStoreReader,
    index: Arc<RwLock<HashMap<String, CommandOffset>>>,
    current_gen: u64,
    uncompacted: u64,
}

impl KvStoreWriter {
    fn new(
        path: Arc<PathBuf>,
        writer: BufWriter<File>,
        reader: KvStoreReader,
        index: Arc<RwLock<HashMap<String, CommandOffset>>>,
        current_gen: u64,
    ) -> Result<Self> {
        Ok(KvStoreWriter {
            path,
            writer: PosBufWriter::new(writer)?,
            reader,
            index,
            current_gen,
            uncompacted: 0,
        })
    }

    fn set(&mut self, key: String, value: String) -> Result<()> {
        let command = Command::Set {
            key: key.clone(),
            value,
        };

        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &command)?;
        self.writer.flush()?;

        {
            let new_pos = self.writer.pos;
            let offset = CommandOffset::from((self.current_gen, pos..new_pos));
            let mut index = self.index.write().unwrap();
            if let Some(offset) = index.insert(key, offset) {
                self.uncompacted += offset.len;
            }
        }

        if self.uncompacted >= COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    fn remove(&mut self, key: String) -> Result<()> {
        if !self.index.read().unwrap().contains_key(&key) {
            Err(KvsError::KeyNotFound)
        } else {
            let command = Command::Remove { key: key.clone() };

            serde_json::to_writer(&mut self.writer, &command)?;
            self.writer.flush()?;

            let offset = self
                .index
                .write()
                .unwrap()
                .remove(&key)
                .expect("Unreachable: key not found");
            self.uncompacted += offset.len;

            if self.uncompacted >= COMPACTION_THRESHOLD {
                self.compact()?;
            }

            Ok(())
        }
    }

    fn compact(&mut self) -> Result<()> {
        let (compact_writer, compact_reader) =
            new_db_log(&db_path(&self.path, self.current_gen + 1))?;
        let (new_writer, new_reader) = new_db_log(&db_path(&self.path, self.current_gen + 2))?;
        let mut compact_writer = PosBufWriter::new(compact_writer)?;

        let current_gen = self.current_gen + 2;
        self.current_gen = current_gen;
        self.writer = PosBufWriter::new(new_writer)?;
        self.reader
            .add_reader(&(self.current_gen - 1), compact_reader);
        self.reader.add_reader(&self.current_gen, new_reader);

        for (_, value) in self.index.write().unwrap().iter_mut() {
            let CommandOffset { gen, pos, len } = value;
            let buffer = self
                .reader
                .read(&gen.clone(), |reader| -> Result<Vec<u8>> {
                    reader.seek(SeekFrom::Start(*pos))?;
                    let mut buffer = vec![0; *len as usize];
                    reader.read_exact(&mut buffer)?;

                    *pos = compact_writer.pos;
                    *gen = current_gen - 1;
                    Ok(buffer)
                })?;

            compact_writer.write_all(&buffer)?;
        }
        compact_writer.flush()?;

        let stale_gens = generations(&self.path)?
            .into_iter()
            .filter(|gen| *gen <= self.current_gen - 2)
            .collect::<Vec<u64>>();

        for ref gen in stale_gens {
            let path = db_path(&self.path, *gen);
            self.reader.remove_reader(gen);
            fs::remove_file(path)?;
        }

        Ok(())
    }
}

fn db_path(path: &PathBuf, gen: u64) -> PathBuf {
    let file_name = format!("{}.Error", gen);
    path.join(file_name)
}

fn new_db_log(path: &PathBuf) -> Result<(BufWriter<File>, BufReader<File>)> {
    let file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(&path)?;

    let writer = BufWriter::new(file.try_clone()?);
    let reader = BufReader::new(file);

    Ok((writer, reader))
}

fn generations(path: &PathBuf) -> Result<Vec<u64>> {
    let mut gens = fs::read_dir(path)?
        .flat_map(|entry| -> Result<_> { Ok(entry?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("Error".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|str| str.trim_end_matches(".Error"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect::<Vec<u64>>();

    gens.sort_unstable();
    Ok(gens)
}

fn load_index(
    gen: u64,
    reader: &mut BufReader<File>,
    index: &mut HashMap<String, CommandOffset>,
) -> Result<()> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;

        match cmd? {
            Command::Set { key, value: _ } => {
                index.insert(key, From::from((gen, pos..new_pos)));
            }
            Command::Remove { key } => {
                index.remove(&key);
            }
        }

        pos = new_pos;
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

#[derive(Debug)]
struct CommandOffset {
    gen: u64,
    pos: u64,
    len: u64,
}

impl From<(u64, Range<u64>)> for CommandOffset {
    fn from((gen, range): (u64, Range<u64>)) -> Self {
        CommandOffset {
            gen,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}
