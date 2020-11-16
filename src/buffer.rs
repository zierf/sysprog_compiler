//! Simple buffer to consume single characters.

use std::io::prelude::Read;
use std::io;
use std::cell::RefCell;

/// Buffer for consuming single characters.
/// Can take any Type with the `Read` trait as source.
///
/// Is divided into two equal chunks and can take back
/// already read characters (up to the size of a chunk).
pub struct CharBuffer<R> {
    /// Readable source to buffer.
    source: RefCell<R>,
    /// Left half of the buffer.
    buffer_left: Box<[u8]>,
    /// Right half of the buffer.
    buffer_right: Box<[u8]>,
    /// Size of one half.
    chunk_size: usize,
    /// Flag, if characters were taken back across the halves.
    back_half: bool,
    /// Number of characters withdrawn.
    withdrawn: usize,
    /// Position within the entire buffer.
    position: usize,
    /// Amount of consumed characters.
    consumed: usize,
    /// Total number of characters read.
    /// Increases as long as more characters can be read.
    end: usize,
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
    /// ```no_run
    /// use sysprog_compiler::CharBuffer;
    /// use std::fs::File;
    ///
    /// let file = File::open("input_file.txt").expect("Failed to open File!");
    /// let mut reader = CharBuffer::new(file, 4096);
    /// ```
    pub fn new(source: R, chunk_size: usize) -> CharBuffer<R> {
        let buffer_left = vec![0; chunk_size];
        let buffer_right = vec![0; chunk_size];

        CharBuffer {
            source: RefCell::new(source),
            buffer_left: buffer_left.into_boxed_slice(),
            buffer_right: buffer_right.into_boxed_slice(),
            back_half: false,
            withdrawn: 0,
            chunk_size,
            position: chunk_size * 2, // resetting before first take
            consumed: 0,
            end: 0,
        }
    }

    /// Loads the next chunk.
    /// Fills the other half, depending on the current position.
    fn read_chunk(&mut self) -> io::Result<usize> {
        let mut handle = self.source.get_mut().take(self.chunk_size as u64);

        let loaded = match self.position {
            x if x < self.chunk_size => handle.read(&mut self.buffer_right)?,
            _                        => handle.read(&mut self.buffer_left)?,
        };

        Result::Ok(loaded)
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
        if self.position < self.chunk_size && (self.position + 1) >= self.chunk_size {
            if !self.back_half {
                self.end += self.read_chunk()?;
            } else {
                self.back_half = false;
            }
            self.position += 1;
        } else if (self.position + 1) >= self.capacity() {
            if !self.back_half {
                self.end += self.read_chunk()?;
            } else {
                self.back_half = false;
            }
            self.position = 0;
        } else if self.consumed < self.end {
            self.position += 1;
        }

        if self.consumed >= self.end {
            return io::Result::Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Reached end of Buffer!"));
        }

        self.consumed +=1;

        if self.withdrawn > 0 {
            self.withdrawn -= 1;
        }

        if self.position < self.chunk_size {
            io::Result::Ok(self.buffer_left[self.position])
        } else {
            io::Result::Ok(self.buffer_right[self.position % self.chunk_size])
        }
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
    /// Returns the total capacity of the buffer.
    ///
    /// ```
    /// # use sysprog_compiler::CharBuffer;
    /// # let input = std::io::empty();
    /// let mut reader = CharBuffer::new(input, 4096);
    /// println!("{}", reader.capacity());
    /// ```
    pub fn capacity(&self) -> usize {
        self.chunk_size * 2
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
        if amount > self.chunk_size {
            return io::Result::Err(io::Error::new(io::ErrorKind::PermissionDenied, "Can not take back more characters than the size of a chunk!"));
        } else if amount > self.consumed {
            return io::Result::Err(io::Error::new(io::ErrorKind::PermissionDenied, "Can not take back more characters than already consumed!"));
        }

        for _ in (0..amount).rev() {
            if (self.withdrawn + 1) > self.chunk_size {
                return io::Result::Err(io::Error::new(io::ErrorKind::PermissionDenied, "Can not take back more characters than the size of a chunk!"));
            }

            if self.position == 0 {
                self.position = self.capacity();
                self.back_half = true;
            } else if self.position == self.chunk_size {
                self.back_half = true;
            }

            self.withdrawn += 1;
            self.consumed -= 1;
            self.position -= 1;
        }

        io::Result::Ok(self.position)
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
        let position = if self.position < self.capacity() {
            self.position
        } else {
            0
        };

        fmt.debug_struct("CharBuffer")
            .field("source", &self.source.borrow())
            .field("left", &format!("{:X?}", &self.buffer_left))
            .field("right", &format!("{:X?}", &self.buffer_right))
            .field("position", &format_args!("{}/{}", position, self.capacity()))
            .finish()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

}
