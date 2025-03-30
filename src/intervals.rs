use nexosim::time::MonotonicTime;

pub type Interval = (MonotonicTime, MonotonicTime);

pub const LT: fn(Interval, Interval) -> bool = |a, b| (a.1 < b.0);
pub const GT: fn(Interval, Interval) -> bool = |a,b| (a.0 > b.1);
pub const OVERLAP: fn(Interval, Interval) -> bool = |a, b| (!LT(a, b) && !GT(a, b));
