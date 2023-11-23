#[inline]
pub fn check<T, E: std::fmt::Display>(r: Result<T, E>) -> T {
    match r {
        Ok(t) => t,
        Err(e) => {
            idlc_errors::unrecoverable!("{e}");
        }
    }
}
