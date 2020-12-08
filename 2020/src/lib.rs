pub mod io {
    use std::io::BufRead;

    pub struct EmptyLineSeparated<R: BufRead> {
        in_record: bool,
        buffer: String,
        inner: R,
        eof: bool,
    }

    impl<R: BufRead> EmptyLineSeparated<R> {
        pub fn new(input: R) -> Self {
            Self {
                in_record: true,
                buffer: String::new(),
                inner: input,
                eof: false,
            }
        }

        pub fn read_next(&mut self) -> Result<Option<&str>, std::io::Error> {
            if self.eof {
                // originally thought might still have some unprocessed
                // but the only way we get here is to have an read == 0
                // which in turn would not have put anything in the buffer.
                return Ok(None);
            }

            loop {
                let before = if !self.in_record {
                    // avoid a drain in the else branch of buf.is_empty()
                    // by clearing the buffer before reading to it
                    self.buffer.clear();
                    0
                } else {
                    self.buffer.len()
                };

                let read = self.inner.read_line(&mut self.buffer)?;
                let pre_trim = &self.buffer[before..];
                let buf = pre_trim.trim();

                if buf.is_empty() {
                    assert!(self.in_record);
                    self.in_record = false;
                    self.eof = read == 0;

                    // this seemed a bit tricky but for our input, this will hold because of this there
                    // does not need to be a buffer.drain(..before) in the else branch
                    assert_eq!(pre_trim, if self.eof { "" } else { "\n" });
                    return Ok(Some(&self.buffer[..before]));
                } else {
                    self.in_record = true;
                }
            }
        }
    }
}
