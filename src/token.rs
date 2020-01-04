use half::f16;

#[derive(Clone, Debug, PartialEq)]
pub enum Token<'a> {
    /// Major type 0: an unsigned integer.
    Unsigned(u64, Encoding),
    /// Major type 1: a negative integer.
    ///
    /// The numeric value is -1 minus the given value.
    Negative(u64, Encoding),
    /// Major type 2: a byte string.
    Bytes(&'a [u8], Encoding),
    /// Major type 3: a text string.
    Text(&'a str, Encoding),
    Tag(u64),
    SimpleValue(u8),
    Half(f16),
    Single(f32),
    Double(f64),
    StartBytes,
    StartText,
    /// Major type 4: an array.
    StartArray(Option<(u64, Encoding)>),
    /// Major type 5: a map.
    StartMap(Option<(u64, Encoding)>),
    Stop,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Encoding {
    SameByte,
    OneByte,
    TwoBytes,
    FourBytes,
    EightBytes,
}
