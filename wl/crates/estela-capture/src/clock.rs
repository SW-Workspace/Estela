use estela_model::Time;

pub trait Clock: Send + Sync {
    fn now(&self) -> Time;
}
