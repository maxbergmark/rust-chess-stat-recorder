pub trait AndThenErr<U, E> {
    fn and_then_err(self, f: impl Fn(&E) -> std::result::Result<U, E>) -> Self;
}

impl<T, U, E> AndThenErr<U, E> for std::result::Result<T, E> {
    fn and_then_err(self, f: impl FnOnce(&E) -> std::result::Result<U, E>) -> Self {
        self.map_err(|e| f(&e).err().unwrap_or(e))
    }
}
