use common::Ptr;

const PAGE_SIZE: usize = 128;

#[derive(Default, Debug)]
struct FreePlace {
    vec: usize,
    index: usize,
}

#[derive(Default, Debug)]
pub struct Book<T> {
    freelist: Vec<FreePlace>,
    static_page: Vec<Ptr<T>>,
    book: Vec<Vec<Option<Ptr<T>>>>,
}

impl<T> Book<T> {
    pub fn static_page(&mut self) -> &mut Vec<Ptr<T>> {
        &mut self.static_page
    }

    pub fn insert(&mut self, ptr: &Ptr<T>) {
        if self.book.len() > 0 && self.book.last().unwrap().len() < PAGE_SIZE {
            self.book.last_mut().unwrap().push(Some(ptr.clone()));
        } else if let Some(place) = self.freelist.pop() {
            self.book[place.vec][place.index] = Some(ptr.clone());
        } else {
            self.book.push(Vec::with_capacity(PAGE_SIZE));
            self.book.last_mut().unwrap().push(Some(ptr.clone()));
        }
    }

    pub fn remove(&mut self, ptr: Ptr<T>) {
        // NOTE Book + Argument = 2
        if ptr.strong_count() != 2 || ptr.weak_count() != 0 {
            panic!("Tried to free a object that is still beeing used.");
        }
        for (i, v) in self.book.iter_mut().enumerate() {
            for (j, mut o) in v.iter_mut().enumerate() {
                if o.is_some() && o.as_ref().unwrap() == &ptr {
                    *o = None;
                    self.freelist.push(FreePlace { vec: i, index: j });
                    return;
                }
            }
        }
        panic!("Tried to remove non static_page object.");
    }
}
