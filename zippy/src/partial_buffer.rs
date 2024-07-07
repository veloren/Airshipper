#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryDescription {
    start: usize,
    size: usize,
}

#[derive(Debug)]
pub struct Entry {
    start: usize,
    data: bytes::Bytes,
}

#[derive(Default)]
pub struct PartialBuffer {
    content: Vec<Entry>,
}

impl PartialBuffer {
    pub fn read(
        &self,
        start: usize,
        size: usize,
    ) -> Result<Entry, Vec<EntryDescription>> {
        let request = EntryDescription { start, size };

        for e in self.content.iter() {
            if request.is_inside(&e) {
                let local_start = start - e.start;
                let data = e.data.slice(local_start..local_start + size);
                return Ok(Entry { data, start });
            }
        }

        let c: Vec<_> = self.content.iter().collect();
        let missing = request.missing(&c);

        Err(missing)
    }

    pub fn store(&mut self, entry: Entry) {
        self.content.push(entry);
        while self.maintanance() {}
    }

    /// try to optimize store and get rid of redundant info
    fn maintanance(&mut self) -> bool {
        self.content.sort_by_key(|c| c.start);
        for i in 0..self.content.len() - 1 {
            let e = &self.content[i];
            let e2 = &self.content[i + 1];

            if e.one_after_last() >= e2.start {
                self.content[i] = e.concat(e2);
                self.content.remove(i + 1);
                return true;
            }
        }
        false
    }
}

impl Entry {
    fn concat(&self, other: &Entry) -> Self {
        use bytes::BufMut;
        let bytes_from_first = self.data.len();
        let bytes_from_second = other.one_after_last() - self.one_after_last();
        let mut bytes =
            bytes::BytesMut::with_capacity(bytes_from_first + bytes_from_second);
        bytes.put(self.data.slice(0..self.data.len()));
        bytes.put(
            other
                .data
                .slice(other.data.len() - bytes_from_second..other.data.len()),
        );

        Self {
            data: bytes.freeze(),
            start: self.start,
        }
    }
}

impl EntryDescription {
    pub fn new(start: usize, size: usize) -> Self {
        Self { start, size }
    }
}

trait Boundary {
    fn start(&self) -> usize;
    fn one_after_last(&self) -> usize;
    fn is_inside<T: Boundary>(&self, other: &T) -> bool {
        self.start() >= other.start() && self.one_after_last() <= other.one_after_last()
    }
    /// removes other values from self, returns either length 0 or 1 or 2
    fn remove_from<T: Boundary>(&self, other: &T) -> Vec<EntryDescription> {
        if self.is_inside(other) {
            return vec![];
        }
        let mut result = vec![];
        if self.start() < other.start() && self.one_after_last() >= other.start() {
            result.push(EntryDescription {
                start: self.start(),
                size: other.start() - self.start(),
            })
        }
        if self.one_after_last() > other.one_after_last()
            && self.start() <= other.one_after_last()
        {
            result.push(EntryDescription {
                start: other.one_after_last(),
                size: self.one_after_last() - other.one_after_last(),
            })
        }
        if result.is_empty() {
            return vec![EntryDescription {
                start: self.start(),
                size: self.one_after_last() - self.start(),
            }];
        }

        result
    }

    /// check what parts are missing, compared to a list of existing
    fn missing<T: Boundary>(&self, others: &[T]) -> Vec<EntryDescription> {
        let mut others: Vec<_> = others.iter().collect();
        others.sort_by_key(|e| e.start());
        let own = EntryDescription {
            start: self.start(),
            size: self.one_after_last() - self.start(),
        };
        let mut missing = vec![own];
        'outer: while let Some(m) = missing.pop() {
            println!("picked: {:?}", &m);

            for o in &others {
                println!("other: {}-{}", &o.start(), &o.one_after_last());
                let mut next = m.remove_from(*o);
                println!("next: {:?}", &next);
                if next != vec![m] {
                    missing.append(&mut next);
                    println!("end {:?}", &missing);
                    continue 'outer;
                }
            }
            missing.push(m);
            break;
        }
        println!("end2 {:?}", &missing);
        missing
    }
}

impl Boundary for EntryDescription {
    fn start(&self) -> usize {
        self.start
    }

    fn one_after_last(&self) -> usize {
        self.start + self.size
    }
}

impl Boundary for &Entry {
    fn start(&self) -> usize {
        self.start
    }

    fn one_after_last(&self) -> usize {
        <&Entry as Into<EntryDescription>>::into(self).one_after_last()
    }
}

impl From<&Entry> for EntryDescription {
    fn from(value: &Entry) -> Self {
        EntryDescription {
            start: value.start,
            size: value.data.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[test]
    fn entries() {
        let data1 = Bytes::from("hello w");
        let data2 = Bytes::from("orld");
        let e1 = Entry {
            data: data1,
            start: 0,
        };
        let e2 = Entry {
            data: data2,
            start: 7,
        };
        let new = e1.concat(&e2);
        assert_eq!(new.start, 0);
        assert_eq!(new.data, "hello world");
    }

    #[test]
    fn remove_from() {
        let datas = [
            EntryDescription::new(2, 2),
            EntryDescription::new(5, 1),
            EntryDescription::new(6, 1),
            EntryDescription::new(7, 4),
        ];
        // 1:4
        assert_eq!(EntryDescription::new(1, 4).remove_from(&datas[0]), vec![
            EntryDescription::new(1, 1),
            EntryDescription::new(4, 1)
        ]);
        assert_eq!(EntryDescription::new(1, 4).remove_from(&datas[1]), vec![
            EntryDescription::new(1, 4),
        ]);
        assert_eq!(EntryDescription::new(1, 4).remove_from(&datas[2]), vec![
            EntryDescription::new(1, 4),
        ]);

        // 1:1
        assert_eq!(EntryDescription::new(1, 1).remove_from(&datas[0]), vec![
            EntryDescription::new(1, 1)
        ]);
        assert_eq!(EntryDescription::new(1, 1).remove_from(&datas[1]), vec![
            EntryDescription::new(1, 1),
        ]);

        // 4:1
        assert_eq!(EntryDescription::new(4, 1).remove_from(&datas[0]), vec![
            EntryDescription::new(4, 1)
        ]);
        assert_eq!(EntryDescription::new(4, 1).remove_from(&datas[1]), vec![
            EntryDescription::new(4, 1),
        ]);
        assert_eq!(EntryDescription::new(4, 1).remove_from(&datas[2]), vec![
            EntryDescription::new(4, 1),
        ]);

        // 4:3
        assert_eq!(EntryDescription::new(4, 3).remove_from(&datas[0]), vec![
            EntryDescription::new(4, 3)
        ]);
        assert_eq!(EntryDescription::new(4, 3).remove_from(&datas[1]), vec![
            EntryDescription::new(4, 1),
            EntryDescription::new(6, 1),
        ]);
        assert_eq!(EntryDescription::new(4, 3).remove_from(&datas[2]), vec![
            EntryDescription::new(4, 2),
        ]);

        // 5:1
        assert_eq!(EntryDescription::new(5, 1).remove_from(&datas[0]), vec![
            EntryDescription::new(5, 1)
        ]);
        assert_eq!(EntryDescription::new(5, 1).remove_from(&datas[1]), vec![]);
        assert_eq!(EntryDescription::new(5, 1).remove_from(&datas[2]), vec![
            EntryDescription::new(5, 1),
        ]);
    }

    #[test]
    fn buffer() {
        let mut buffer = PartialBuffer::default();
        buffer.store(Entry {
            data: Bytes::from("efgh"),
            start: 4,
        });
        assert_eq!(buffer.read(0, 4).unwrap_err()[0], EntryDescription {
            start: 0,
            size: 4
        });
        buffer.store(Entry {
            data: Bytes::from("abcd"),
            start: 0,
        });
        assert_eq!(buffer.read(6, 4).unwrap_err()[0], EntryDescription {
            start: 8,
            size: 2,
        });
        buffer.store(Entry {
            data: Bytes::from("mnop"),
            start: 12,
        });
        assert_eq!(buffer.read(8, 4).unwrap_err()[0], EntryDescription {
            start: 8,
            size: 4,
        });
        assert_eq!(buffer.read(6, 8).unwrap_err()[0], EntryDescription {
            start: 8,
            size: 4,
        });
        buffer.store(Entry {
            data: Bytes::from("ijkl"),
            start: 8,
        });
        buffer.store(Entry {
            data: Bytes::from("qrst"),
            start: 16,
        });

        assert_eq!(buffer.content.len(), 1);
        assert_eq!(buffer.content[0].data, "abcdefghijklmnopqrst");
        assert_eq!(buffer.read(0, 4).unwrap().data, "abcd");
        assert_eq!(buffer.read(2, 4).unwrap().data, "cdef");
    }
}
