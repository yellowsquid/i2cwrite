use i2cwrite::Encoding;

struct ScanCode {
    base: char,
    shift: bool,
}

const CODE_TABLE: [Option<(char, bool)>; 128] = [
    None,
    None,
    None,
    None, // 0x04
    None,
    None,
    None,
    None, // 0x08
    None,
    Some(('\t', false)),
    None,
    None, // 0x0c
    None,
    Some(('\n', false)),
    None,
    None, // 0x10
    None,
    None,
    None,
    None, // 0x14
    None,
    None,
    None,
    None, // 0x18
    None,
    None,
    None,
    Some(('\u{1b}', false)), // 0x1c
    None,
    None,
    None,
    None, // 0x20
    Some((' ', false)),
    Some(('1', true)),
    Some(('\'', true)),
    Some(('3', true)), // 0x24
    Some(('4', true)),
    Some(('5', true)),
    Some(('7', true)),
    Some(('\'', false)), // 0x28
    Some(('9', true)),
    Some(('0', true)),
    Some(('8', true)),
    Some(('=', true)), // 0x2c
    Some((',', false)),
    Some(('-', false)),
    Some(('.', false)),
    Some(('/', false)), // 0x30
    Some(('0', false)),
    Some(('1', false)),
    Some(('2', false)),
    Some(('3', false)), // 0x34
    Some(('4', false)),
    Some(('5', false)),
    Some(('6', false)),
    Some(('7', false)), // 0x38
    Some(('8', false)),
    Some(('9', false)),
    Some((';', true)),
    Some((';', false)), // 0x3c
    Some((',', true)),
    Some(('=', false)),
    Some(('.', true)),
    Some(('/', true)), // 0x40
    Some(('2', true)),
    Some(('a', true)),
    Some(('b', true)),
    Some(('c', true)), // 0x44
    Some(('d', true)),
    Some(('e', true)),
    Some(('f', true)),
    Some(('g', true)), // 0x48
    Some(('h', true)),
    Some(('i', true)),
    Some(('j', true)),
    Some(('k', true)), // 0x4c
    Some(('l', true)),
    Some(('m', true)),
    Some(('n', true)),
    Some(('o', true)), // 0x50
    Some(('p', true)),
    Some(('q', true)),
    Some(('r', true)),
    Some(('s', true)), // 0x54
    Some(('t', true)),
    Some(('u', true)),
    Some(('v', true)),
    Some(('w', true)), // 0x58
    Some(('x', true)),
    Some(('y', true)),
    Some(('z', true)),
    Some(('[', false)), // 0x5c
    Some(('\\', false)),
    Some((']', false)),
    Some(('6', true)),
    Some(('-', true)), // 0x60
    Some(('`', false)),
    Some(('a', false)),
    Some(('b', false)),
    Some(('c', false)), // 0x64
    Some(('d', false)),
    Some(('e', false)),
    Some(('f', false)),
    Some(('g', false)), // 0x68
    Some(('h', false)),
    Some(('i', false)),
    Some(('j', false)),
    Some(('k', false)), // 0x6c
    Some(('l', false)),
    Some(('m', false)),
    Some(('n', false)),
    Some(('o', false)), // 0x70
    Some(('p', false)),
    Some(('q', false)),
    Some(('r', false)),
    Some(('s', false)), // 0x74
    Some(('t', false)),
    Some(('u', false)),
    Some(('v', false)),
    Some(('w', false)), // 0x78
    Some(('x', false)),
    Some(('y', false)),
    Some(('z', false)),
    Some(('[', true)), // 0x7c
    Some(('\\', true)),
    Some((']', true)),
    Some(('`', true)),
    Some(('\u{7f}', false)), // 0x80
];

impl ScanCode {
    fn of(byte: u8) -> Option<Self> {
        let index = usize::from(byte);
        if index >= CODE_TABLE.len() {
            None
        } else {
            CODE_TABLE[index].map(|(base, shift)| Self::new(base, shift))
        }
    }

    fn new(base: char, shift: bool) -> Self {
        Self { base, shift }
    }
}

pub enum ScanCodeSet {
    Set2,
}

impl Encoding for ScanCodeSet {
    fn encode(&self, byte: u8) -> Option<Vec<u8>> {
        ScanCode::of(byte).and_then(|code| self.translate(code))
    }
}

impl ScanCodeSet {
    fn translate(&self, code: ScanCode) -> Option<Vec<u8>> {
        let base = self.translate_base(code.base)?;

        match self {
            ScanCodeSet::Set2 => {
                if !code.shift {
                    let mut out = Vec::with_capacity(base.len() * 2 + 1);
                    out.extend(base.iter().clone());
                    out.push(0xf0);
                    out.extend(base.into_iter());
                    Some(out)
                } else {
                    let mut out = Vec::with_capacity(base.len() * 2 + 4);
                    out.push(0x12);
                    out.extend(base.iter().clone());
                    out.push(0xf0);
                    out.extend(base.iter().clone());
                    out.push(0xf0);
                    out.push(0x12);
                    Some(out)
                }
            }
        }
    }

    // Translations grabbed from [here].
    //
    // [here]: http://kbd-project.org/docs/scancodes/scancodes-10.html
    fn translate_base(&self, base: char) -> Option<Vec<u8>> {
        match base {
            '`' => Some(vec![0x0e]),
            '1' => Some(vec![0x16]),
            '2' => Some(vec![0x1e]),
            '3' => Some(vec![0x26]),
            '4' => Some(vec![0x25]),
            '5' => Some(vec![0x2e]),
            '6' => Some(vec![0x36]),
            '7' => Some(vec![0x3d]),
            '8' => Some(vec![0x3e]),
            '9' => Some(vec![0x46]),
            '0' => Some(vec![0x45]),
            '-' => Some(vec![0x4e]),
            '=' => Some(vec![0x55]),
            '\t' => Some(vec![0x0d]),
            'q' => Some(vec![0x15]),
            'w' => Some(vec![0x1d]),
            'e' => Some(vec![0x24]),
            'r' => Some(vec![0x2d]),
            't' => Some(vec![0x2c]),
            'y' => Some(vec![0x35]),
            'u' => Some(vec![0x3c]),
            'i' => Some(vec![0x43]),
            'o' => Some(vec![0x44]),
            'p' => Some(vec![0x4d]),
            '[' => Some(vec![0x54]),
            ']' => Some(vec![0x5b]),
            '\\' => Some(vec![0x5d]),
            'a' => Some(vec![0x1c]),
            's' => Some(vec![0x1b]),
            'd' => Some(vec![0x23]),
            'f' => Some(vec![0x2b]),
            'g' => Some(vec![0x34]),
            'h' => Some(vec![0x33]),
            'j' => Some(vec![0x3b]),
            'k' => Some(vec![0x42]),
            'l' => Some(vec![0x4b]),
            ';' => Some(vec![0x4c]),
            '\'' => Some(vec![0x52]),
            '\n' => Some(vec![0x5a]),
            'z' => Some(vec![0x1a]),
            'x' => Some(vec![0x22]),
            'c' => Some(vec![0x21]),
            'v' => Some(vec![0x2a]),
            'b' => Some(vec![0x32]),
            'n' => Some(vec![0x31]),
            'm' => Some(vec![0x3a]),
            ',' => Some(vec![0x41]),
            '.' => Some(vec![0x49]),
            '/' => Some(vec![0x4a]),
            ' ' => Some(vec![0x29]),
            '\u{7f}' => Some(vec![0xe0, 0x71]),
            '\u{1b}' => Some(vec![0x76]),
            _ => None,
        }
    }
}
