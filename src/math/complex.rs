#[derive(Debug, Clone, Copy)]
pub struct Complex<T> {
    real: T,
    imag: T,
}

impl<T> From<(T, T)> for Complex<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            real: value.0,
            imag: value.1,
        }
    }
}

impl<T> Complex<T> {
    pub fn real(&self) -> &T {
        &self.real
    }

    pub fn imag(&self) -> &T {
        &self.imag
    }
}
