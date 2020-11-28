/// Allows for inserting into a `ReserveVec` at a specific position.
pub struct ReservePos(usize);

#[derive(Debug)]
enum VecItem<T> {
    Value(T),
    Marker,
}

/// A vector wrapper enabling reserving positions in the vector.
#[derive(Debug)]
pub struct ReserveVec<T> {
    vec: Vec<VecItem<T>>,
}

impl<T> ReserveVec<T> {
    pub fn new() -> ReserveVec<T> {
        ReserveVec { vec: Vec::new() }
    }

    /// Identical to a regular vector push.
    pub fn push(&mut self, value: T) {
        self.vec.push(VecItem::Value(value));
    }

    /// Reserve the next position in the vector.
    pub fn reserve_next(&mut self) -> ReservePos {
        let idx = self.vec.len();
        self.vec.push(VecItem::Marker);
        ReservePos(idx)
    }

    /// Insert a value at a previously reserved position.
    pub fn insert_at_reserved(&mut self, pos: ReservePos, value: T) {
        self.vec[pos.0] = VecItem::Value(value);
    }
}

impl<T> Into<Vec<T>> for ReserveVec<T> {
    fn into(self) -> Vec<T> {
        self.vec
            .into_iter()
            .filter_map(|v| match v {
                VecItem::Value(v) => Some(v),
                _ => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_reservations() {
        let mut r = ReserveVec::new();
        r.push(1);
        r.push(2);
        r.push(3);
        let v: Vec<_> = r.into();
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn reservation_at_beginning() {
        let mut r = ReserveVec::new();
        let pos = r.reserve_next();
        r.push(1);
        r.push(2);
        r.insert_at_reserved(pos, 3);
        let v: Vec<_> = r.into();
        assert_eq!(v, vec![3, 1, 2]);
    }

    #[test]
    fn multiple_reservations() {
        let mut r = ReserveVec::new();
        let p1 = r.reserve_next();
        r.push(1);
        let p2 = r.reserve_next();
        let p3 = r.reserve_next();
        r.push(2);
        let p4 = r.reserve_next();
        r.insert_at_reserved(p1, 3);
        r.insert_at_reserved(p2, 4);
        r.insert_at_reserved(p3, 5);
        r.insert_at_reserved(p4, 6);
        let v: Vec<_> = r.into();
        assert_eq!(v, vec![3, 1, 4, 5, 2, 6]);
    }
}
