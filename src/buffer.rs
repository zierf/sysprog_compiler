//! Simple buffer to consume single characters.

use std::io::prelude::Read;
use std::io;
use std::cell::RefCell;


/// Chunk data for left and right half of a Buffer.
#[derive(Debug)]
struct Chunks {
    /// Size of one half.
    size: usize,
    /// Left half of the buffer.
    left: Box<[u8]>,
    /// Right half of the buffer.
    right: Box<[u8]>,
}

/// Buffer for consuming single characters.
/// Can take any Type with the `Read` trait as source.
///
/// Is divided into two equal chunks and can take back
/// already read characters (up to the size of a chunk).
pub struct CharBuffer<R> {
    /// Readable source to buffer.
    source: RefCell<R>,
    /// Buffer consisting of two chunks (left and right half).
    chunks: Chunks,
    /// Amount of consumed characters.
    consumed: usize,
    /// Total number of characters read.
    /// Increases while reading more characters.
    loaded: usize,
    /// Number of characters withdrawn.
    withdrawn: usize,
}

impl<R> CharBuffer<R>
where
    R: Read
{
    /// Create a new Buffer.
    ///
    /// Takes any type implementing the `Read` trait
    /// like an instance of a `File`.
    ///
    /// Chunk size has to be at least one or greater.
    ///
    /// ```no_run
    /// use sysprog_compiler::CharBuffer;
    /// use std::fs::File;
    ///
    /// let file = File::open("input_file.txt").expect("Failed to open File!");
    /// let mut reader = CharBuffer::new(file, 4096);
    /// ```
    pub fn new(source: R, chunk_size: usize) -> CharBuffer<R> {
        if chunk_size < 1 {
            panic!("The block size must be greater than or equal to one!");
        }

        let buffer_left = vec![0; chunk_size];
        let buffer_right = vec![0; chunk_size];

        CharBuffer {
            source: RefCell::new(source),
            chunks: Chunks {
                size: chunk_size,
                left: buffer_left.into_boxed_slice(),
                right: buffer_right.into_boxed_slice(),
            },
            consumed: 0,
            loaded: 0,
            withdrawn: 0,
        }
    }

    /// Loads the next chunk.
    /// Fills the other half, depending on the current position.
    fn load_chunk(&mut self) -> io::Result<usize> {
        let position = self.position();

        let mut handle = self.source.get_mut().take(self.chunks.size as u64);

        let loaded = match position {
            x if x < self.chunks.size => handle.read(&mut self.chunks.left)?,
            _                         => handle.read(&mut self.chunks.right)?,
        };

        Result::Ok(loaded)
    }

    /// Read a specific position from buffer.
    fn read_position(&mut self, position: usize) -> io::Result<u8> {
        if position >= self.capacity() {
            return io::Result::Err(io::Error::new(io::ErrorKind::NotFound, "The specified position is greater than the capacity of the buffer!!"));
        }

        if self.position() < self.chunks.size {
            io::Result::Ok(self.chunks.left[position])
        } else {
            io::Result::Ok(self.chunks.right[position % self.chunks.size])
        }
    }

    /// Reads the next byte from the buffer.
    ///
    /// ```
    /// # use sysprog_compiler::CharBuffer;
    /// # let input = std::io::empty();
    /// let mut reader = CharBuffer::new(input, 4096);
    ///
    /// while let Ok(byte) = reader.take_byte() {
    ///     print!("{:#X?} ", byte);
    /// }
    /// ```
    pub fn take_byte(&mut self) -> io::Result<u8> {
        let position = self.position();

        if (self.withdrawn == 0) && (position == 0 || position == self.chunks.size) {
            self.loaded += self.load_chunk()?;
        }

        if self.consumed >= self.loaded {
            return io::Result::Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Reached end of Buffer!"));
        }

        if self.withdrawn > 0 {
            self.withdrawn -= 1;
        }

        let byte = self.read_position(position)?;

        self.consumed +=1;
        io::Result::Ok(byte)
    }

    /// Reads the next byte as character from the buffer.
    /// Only ASCII characters are currently supported.
    ///
    /// ```
    /// # use sysprog_compiler::CharBuffer;
    /// # let input = std::io::empty();
    /// let mut reader = CharBuffer::new(input, 4096);
    ///
    /// while let Ok(character) = reader.take_char() {
    ///     print!("{}", character);
    /// }
    /// ```
    pub fn take_char(&mut self) -> io::Result<char> {
        let byte = self.take_byte()?;

        if !byte.is_ascii() {
            return io::Result::Err(io::Error::new(io::ErrorKind::InvalidInput, "Not a valid ASCII character!"));
        }

        io::Result::Ok(byte as char)
    }
}

impl<R> CharBuffer<R> {
    /// Returns the current position across both buffer chunks.
    pub fn position(&self) -> usize {
        self.consumed % self.capacity()
    }

    /// Returns the total capacity of the buffer.
    ///
    /// ```
    /// # use sysprog_compiler::CharBuffer;
    /// # let input = std::io::empty();
    /// let mut reader = CharBuffer::new(input, 4096);
    /// println!("{}", reader.capacity());
    /// ```
    pub fn capacity(&self) -> usize {
        self.chunks.size * 2
    }

    /// Takes back the specified number of characters.
    /// Characters that have already been read can then be read again.
    /// Can't take back more characters than the size of a chunk.
    ///
    /// ```no_run
    /// # fn main() -> std::io::Result<()> {
    /// # use sysprog_compiler::CharBuffer;
    /// # let input = std::io::empty();
    /// let mut reader = CharBuffer::new(input, 4096);
    ///
    /// for _i in (0..2048).rev() {
    ///     let character = reader.take_char()?;
    ///     print!("{}", character);
    /// }
    ///
    /// // maximum is size of one chunk (half the buffer size)
    /// reader.take_back(2048)?;
    ///
    /// while let Ok(character) = reader.take_char() {
    ///     print!("{}", character);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn take_back(&mut self, amount: usize) -> io::Result<usize> {
        let message_too_many = "Can not take back more characters than the size of a chunk!";

        if amount > self.consumed {
            return io::Result::Err(io::Error::new(io::ErrorKind::PermissionDenied, "Can not take back more characters than already consumed!"));
        } else if amount > self.chunks.size {
            return io::Result::Err(io::Error::new(io::ErrorKind::PermissionDenied, message_too_many));
        }

        for _ in (0..amount).rev() {
            if (self.withdrawn + 1) > self.chunks.size {
                return io::Result::Err(io::Error::new(io::ErrorKind::PermissionDenied, message_too_many));
            }

            self.consumed -= 1;
            self.withdrawn += 1;
        }

        io::Result::Ok(self.position())
    }

}

impl<R> std::fmt::Debug for CharBuffer<R>
where
    R: std::fmt::Debug
{
    /// Format an output for debugging.
    ///
    /// ```
    /// # use sysprog_compiler::CharBuffer;
    /// # let input = std::io::empty();
    /// let mut reader = CharBuffer::new(input, 4096);
    /// println!("Buffer {:#?}\n", reader);
    /// ```
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("CharBuffer")
            .field("source", &format!("{:?}", &self.source.borrow()))
            .field("left", &format!("{:02X?}", &self.chunks.left))
            .field("right", &format!("{:02X?}", &self.chunks.right))
            .field("position", &format_args!("{} ({} Positions)", self.position(), self.capacity()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use test::Bencher;

    /// Create a string with printable ASCII characters.
    /// Also contains the linebreak.
    fn create_ascii_string() -> std::string::String {
        let mut input = String::new();
        input.push('\n');

        for keycode in 32..=126 {
            input.push(std::char::from_u32(keycode).unwrap());
        }

        input
    }

    #[test]
    fn create_buffer() {
        let input = std::io::empty();
        CharBuffer::new(input, 32);
    }

    #[test]
    #[should_panic]
    fn chunk_size_zero() {
        let input = std::io::empty();
        CharBuffer::new(input, 0);
    }

    #[test]
    fn get_capacity() {
        let mut i = 1;

        while i <= 4096 {
            let input = std::io::empty();
            assert_eq!(CharBuffer::new(input, i).capacity(), (i * 2));
            i *= 2;
        }
    }

    #[test]
    fn format_debug() {
        let input = std::io::empty();
        let reader = CharBuffer::new(input, 32);
        format!("Buffer {:#?}\n", reader);
    }

    #[test]
    fn take_bytes() {
        let input = create_ascii_string();
        let mut reader = CharBuffer::new(input.as_bytes(), 8);

        for input_char in input.chars() {
            let byte = reader.take_byte().unwrap();
            assert_eq!(byte as char, input_char);
        }
    }

    #[test]
    fn take_chars() {
        let input = create_ascii_string();
        let mut reader = CharBuffer::new(input.as_bytes(), 8);

        for input_char in input.chars() {
            let character = reader.take_char().unwrap();
            assert_eq!(character, input_char);
        }
    }

    #[test]
    fn take_back_chars() {
        let input = create_ascii_string();
        let mut reader = CharBuffer::new(input.as_bytes(), 8);

        // consume one and take it back
        reader.take_char().unwrap();
        reader.take_back(1).unwrap();

        // consume the first eight characters (position 0-7)
        for _i in 0..8 {
            reader.take_char().unwrap();
        }

        reader.take_back(8).unwrap();

        // consume the first eight characters again
        for i in 0..8 {
            assert_eq!(input.chars().nth(i).unwrap(), reader.take_char().unwrap());
        }

        // jump into right chunk (position 8) and go back in left half
        reader.take_char().unwrap();
        reader.take_back(8).unwrap();

        for i in 0..8 {
            assert_eq!(input.chars().nth(i + 1).unwrap(), reader.take_char().unwrap());
        }

        // jump into left chunk again (position 0)
        for _i in 0..8 {
            reader.take_char().unwrap();
        }

        reader.take_back(8).unwrap();

        for i in 0..8 {
            // position 8 was already consumed, continue with 9th in reference string
            assert_eq!(input.chars().nth(i + 9).unwrap(), reader.take_char().unwrap());
        }
    }

    #[test]
    fn take_back_too_many() {
        let input = create_ascii_string();
        let mut reader = CharBuffer::new(input.as_bytes(), 8);

        // not enough consumed
        assert_eq!(reader.take_back(1).unwrap_err().kind(), io::ErrorKind::PermissionDenied);

        // consume some characters
        for _i in 0..8 {
            reader.take_char().unwrap();
        }

        // taking back more than a chunk can contain (in one step)
        assert_eq!(reader.take_back(9).unwrap_err().kind(), io::ErrorKind::PermissionDenied);

        // taking back more than a chunk can contain (in multiple steps)
        for _i in (0..8).rev() {
            reader.take_back(1).unwrap();
        }

        assert_eq!(reader.take_back(1).unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn read_ascii() {
        let input = create_ascii_string();

        let mut reader = CharBuffer::new(input.as_bytes(), 8);

        while let Ok(character) = reader.take_char() {
            assert!(character.is_ascii());
        }
    }

    #[test]
    #[should_panic]
    fn read_utf8() {
        let input = "çêéèÇÉÈÊ".as_bytes();
        let mut reader = CharBuffer::new(input, 8);

        reader.take_char().unwrap();
    }

    #[test]
    fn read_small_file() {
        let file = std::fs::File::open("tests/buffer/input.txt").unwrap();

        let mut reader = CharBuffer::new(file, 8);
        let mut characters = String::new();

        while let Ok(character) = reader.take_char() {
            characters.push(character);
        }

        assert_eq!(characters, String::from("abcdefghijklmno\nABCDEFGHIJKLMNO\n012345\n"));
    }

    #[test]
    fn read_big_file() {
        let bible_path = "tests/buffer/bible.txt";
        let bible_text: String = std::fs::read_to_string(bible_path).unwrap().parse().unwrap();

        let file = std::fs::File::open(bible_path).unwrap();
        let mut reader = CharBuffer::new(file, 4096);

        let mut characters = String::new();

        while let Ok(character) = reader.take_char() {
            characters.push(character);
        }

        assert_eq!(characters, bible_text);
    }

    #[bench]
    fn bench_bible_std(bencher: &mut Bencher) {
        let bible_path = "tests/buffer/bible.txt";
        bencher.iter(|| {
            let bible_text: String = std::fs::read_to_string(bible_path).unwrap().parse().unwrap();

            bible_text
        });
    }

    #[bench]
    fn bench_bible_charbuffer(bencher: &mut Bencher) {
        let bible_path = "tests/buffer/bible.txt";

        bencher.iter(|| {
            let file = std::fs::File::open(bible_path).unwrap();
            let mut reader = CharBuffer::new(file, 4096);

            let mut characters = String::new();

            while let Ok(character) = reader.take_char() {
                characters.push(character);
            }

            characters
        });
    }

}
