pub enum Command {
    Get(String),
    Put(String, String),
    Invalid(String),
}
