pub struct IStream<'a> {
    file_name: &'a String,
    input: &'a Vec<u8>,
    pub row: usize,
    pub col: usize,
    idx: usize,
    line_idx: usize,
    pub empty: bool,
}

impl IStream<'_> {
    pub fn next(&mut self) -> u8 {
        let nchar: u8 = self.input[self.idx];
        self.idx += 1;
        if nchar == b'\n' {
            self.row += 1;
            self.col = 0;
            self.line_idx = self.idx;
        } else {
            self.col += 1;
        }
        nchar
    }

    pub fn len(&self) -> usize {
        self.input.len()
    }

    pub fn peek(&self, amount: isize) -> Option<u8> {
        let idx: usize = (self.idx as isize + amount) as usize;
        if idx >= self.len() {
            None
        } else {
            Some(self.input[idx])
        }
    }

    pub fn get_current_line(&self) -> &str {
        let line = &self.input[self.line_idx..];
        let line_end = line.iter().position(|&c| c == b'\n').unwrap();
        std::str::from_utf8(&line[..line_end]).unwrap()
    }

    pub fn take_to_c(&mut self, c: u8) {
        while self.next() != c {}
    }

    pub fn take_while(&mut self, f: impl Fn(u8, Option<&mut IStream>) -> bool) -> Vec<u8> {
        let mut chars: Vec<u8> = Vec::new();
        loop {
            match self.peek(0) {
                None => {
                    self.empty = true;
                    break;
                },
                Some(c) => {
                    if f(c, Some(self)) {
                        chars.push(c);
                        self.next();
                    } else {
                        break
                    }
                }
            }
        }
        return chars;
    }

    pub fn err(&self) {
        println!("Could not parse: {}", self.file_name);
        let row_s = format!("{}", self.row);
        println!("{}: {}", row_s, self.get_current_line());
        for _ in 0..self.col + row_s.len() + 2 {
            print!(" ")
        }
        print!("^");
        println!()
    }

    pub fn new<'a>(input: &'a Vec<u8>, file_name: &'a String) -> IStream<'a> {
        IStream {
            file_name,
            input,
            row: 0,
            col: 0,
            idx: 0,
            line_idx: 0,
            empty: false,
        }
    }
}