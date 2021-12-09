pub struct Spawn<T>(pub &'static str, pub T);
pub struct Abort(pub &'static str);
pub struct Wait;
