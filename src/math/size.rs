pub struct Size<T> {
    w: T,
    h: T,
}

impl<T> From<(T, T)> for Size<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            w: value.0,
            h: value.1,
        }
    }
}

impl<T> Size<T> {
    pub fn w(&self) -> &T {
        &self.w
    }
    pub fn h(&self) -> &T {
        &self.h
    }
}
