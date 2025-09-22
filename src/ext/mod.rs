pub trait _From<T> {
    fn _from(t: T) -> Self;
}

pub trait _Into<T> {
    fn _into(&self) -> T;
}

//// u64 <-> bool
//// (!) extend to T: PrimInt
impl _From<u64> for bool {
    fn _from(t: u64) -> Self {
        if t == 0 {
            return false;
        }
        true
    }
}

impl _Into<u64> for bool {
    fn _into(&self) -> u64 {
        if *self {
            return 0b1;
        }
        0b0
    }
}

/// u32 <-> bool
impl _From<u32> for bool {
    fn _from(t: u32) -> Self {
        if t == 0 { false } else { true }
    }
}

impl _Into<u32> for bool {
    fn _into(&self) -> u32 {
        if *self { 0b1 } else { 0b0 }
    }
}