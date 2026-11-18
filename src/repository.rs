#![allow(dead_code)]
use std::collections::HashMap;
use std::hash::Hash;

/// Identifiable types have a method to return its ID.
pub trait Identifiable {
    type Id: Clone + Eq + Hash + Send + Sync;

    fn id(&self) -> &Self::Id;
}

/// Abstraction to store and manage [`Identifiable`] objects.
pub trait Repository<T: Identifiable>: Send + Sync {
    /// Add a new [`Identifiable`] item to the repository.
    fn add(&mut self, item: T);
    /// Find an [`Identifiable`] item using its ID.
    fn find_by_id(&self, id: &T::Id) -> Option<&T>;
}

/// Simple implementation of [`Repository`] that holds the data in memory.
pub struct InMemoryRepository<T>
where
    T: Identifiable,
{
    entities: HashMap<T::Id, T>,
}

impl<T> InMemoryRepository<T>
where
    T: Identifiable,
{
    /// Create a new [`InMemoryRepository`].
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }
}

impl<T> Repository<T> for InMemoryRepository<T>
where
    T: Identifiable + Send + Sync,
{
    fn add(&mut self, item: T) {
        let id = item.id().clone();
        self.entities.insert(id, item);
    }

    fn find_by_id(&self, id: &T::Id) -> Option<&T> {
        self.entities.get(id)
    }
}
