use rand::seq::SliceRandom;

pub struct AnySlice<T> {
    data: Vec<T>,
}

impl<T> AnySlice<T> {
    pub fn new(vec: Vec<T>) -> Self {
        AnySlice { data: vec }
    }

    pub fn get_data(self) -> Vec<T> {
        self.data
    }

    pub fn to_vec(&self) -> &Vec<T> {
        &self.data
    }

    pub fn set_data(&mut self, data: Vec<T>) {
        self.data = data;
    }

    pub fn set_value(&mut self, index: usize, value: T) -> &mut Self
    where
        T: Clone,
    {
        self.data[index] = value;
        self
    }

    pub fn empty(&self) -> bool {
        self.to_vec().is_empty()
    }

    pub fn not_empty(&self) -> bool {
        !self.to_vec().is_empty()
    }

    pub fn has(&self, value: &Vec<T>) -> bool
    where
        T: PartialEq,
    {
        self.to_vec().iter().any(|item| value.contains(item))
    }

    pub fn not_has(&self, value: &Vec<T>) -> bool
    where
        T: PartialEq,
    {
        !self.has(value)
    }

    pub fn get_value_by_index(&self, index: usize) -> Option<&T> {
        self.to_vec().get(index)
    }

    pub fn get_value_ptr(&self, index: usize) -> Option<*const T> {
        self.to_vec().get(index).map(|value| value as *const T)
    }

    pub fn get_value_default(&self, index: usize, default: T) -> T
    where
        T: Clone,
    {
        self.to_vec().get(index).cloned().unwrap_or(default)
    }

    pub fn get_values(&self, indexes: &[usize]) -> Vec<&T> {
        indexes
            .iter()
            .filter_map(|index| self.to_vec().get(*index))
            .collect()
    }

    pub fn get_values_by_slicer(&self, slicer: &AnySlice<usize>) -> Vec<&T> {
        slicer
            .to_vec()
            .iter()
            .filter_map(|index| self.to_vec().get(*index))
            .collect()
    }

    pub fn first(&self) -> Option<&T> {
        self.to_vec().first()
    }

    pub fn last(&self) -> Option<&T> {
        self.to_vec().last()
    }

    pub fn get_indexes(&self) -> Vec<usize> {
        (0..self.len()).collect()
    }

    pub fn get_index_by_value(&self, value: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        self.to_vec().iter().position(|v| v == value)
    }

    pub fn get_indexes_by_values(&self, values: &Vec<T>) -> Vec<usize>
    where
        T: PartialEq + Clone,
    {
        values
            .iter()
            .map(|value| self.get_index_by_value(value).unwrap())
            .collect()
    }

    pub fn shuffle(&self) -> AnySlice<T>
    where
        T: Clone,
    {
        self.to_vec().clone().shuffle(&mut rand::rng());
        AnySlice {
            data: self.to_vec().clone(),
        }
    }

    pub fn len(&self) -> usize {
        self.to_vec().len()
    }

    pub fn len_without_empty(&self) -> usize
    where
        T: PartialEq + Default + Clone,
    {
        let mut count = 0;
        self.to_vec().iter().for_each(|value| {
            if value != &T::default() {
                count += 1;
            }
        });

        count
    }

    pub fn remove_empty(&mut self) -> &mut Self
    where
        T: PartialEq + Default + Clone,
    {
        let mut data = self.to_vec().clone();
        data.retain(|value| value != &T::default());
        self.set_data(data);
        self
    }

    pub fn append(&mut self, values: Vec<T>) -> &mut Self {
        self.data.extend(values);
        self
    }

    pub fn push(&mut self, value: T) -> &mut Self {
        self.data.push(value);
        self
    }

    pub fn filter(&mut self, predicate: impl Fn(&T) -> bool) -> &mut Self {
        self.data.retain(predicate);
        self
    }

    pub fn all_empty(&self) -> bool
    where
        T: PartialEq + Default + Clone,
    {
        self.to_vec().iter().all(|value| value == &T::default())
    }

    pub fn any_empty(&self) -> bool
    where
        T: PartialEq + Default + Clone,
    {
        self.to_vec().iter().any(|value| value == &T::default())
    }

    pub fn copy(&self) -> Self
    where
        T: Clone,
    {
        AnySlice {
            data: self.data.clone(),
        }
    }

    pub fn chunk(&self, size: usize) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        self.to_vec()
            .chunks(size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }

    pub fn pluck<DST>(&self, func: impl Fn(&T) -> DST) -> Vec<DST> {
        self.to_vec().iter().map(|item| func(item)).collect()
    }

    pub fn intersection(&self, other: &Vec<T>) -> AnySlice<T>
    where
        T: PartialEq + Clone,
    {
        let mut result = Vec::new();
        for item in self.to_vec() {
            if other.to_vec().contains(item) {
                result.push(item.clone());
            }
        }
        AnySlice::new(result)
    }

    pub fn intersection_slicer(&self, other: &AnySlice<T>) -> AnySlice<T>
    where
        T: PartialEq + Clone,
    {
        self.intersection(other.to_vec())
    }

    pub fn difference(&self, other: &Vec<T>) -> AnySlice<T>
    where
        T: PartialEq + Clone,
    {
        let mut result = Vec::new();
        for item in self.to_vec() {
            if !other.to_vec().contains(item) {
                result.push(item.clone());
            }
        }
        AnySlice::new(result)
    }

    pub fn difference_slicer(&self, other: &AnySlice<T>) -> AnySlice<T>
    where
        T: PartialEq + Clone,
    {
        self.difference(other.to_vec())
    }

    pub fn union(&self, other: &Vec<T>) -> AnySlice<T>
    where
        T: PartialEq + Clone,
    {
        let mut result = self.data.clone();
        let to_add: Vec<T> = other
            .iter()
            .filter(|item| !result.contains(item))
            .cloned()
            .collect();
        result.extend(to_add);
        AnySlice::new(result)
    }

    pub fn union_slicer(&self, other: &AnySlice<T>) -> AnySlice<T>
    where
        T: PartialEq + Clone,
    {
        self.union(other.to_vec())
    }

    pub fn remove_by_index(&mut self, index: &usize) -> &Self {
        if *index < self.data.len() {
            self.data.remove(*index);
        }
        self
    }

    pub fn remove_by_indexes(&mut self, indexes: &Vec<usize>) -> &Self {
        let _ = indexes.iter().map(|index| self.data.remove(*index));
        self
    }

    pub fn every(&self, func: impl Fn(usize, &T) -> bool) -> bool {
        for (idx, item) in self.data.iter().enumerate() {
            if !func(idx, item) {
                return false;
            }
        }
        true
    }

    pub fn each(&mut self, func: impl Fn(usize, &T) -> T) -> &Self {
        let mut data = Vec::new();

        self.data.iter().enumerate().for_each(|(idx, item)| {
            data.push(func(idx, item));
        });
        self.data = data;

        self
    }

    pub fn sort(&mut self, func: impl Fn(&T, &T) -> std::cmp::Ordering) -> &mut Self {
        self.data.sort_by(func);
        self
    }

    pub fn clean(&mut self) -> &mut Self {
        self.data.clear();
        self
    }

    pub fn to_string(&self, sep: Option<&str>) -> String
    where
        T: std::fmt::Display,
    {
        self.data
            .iter()
            .map(|item| item.to_string())
            .collect::<Vec<String>>()
            .join(sep.unwrap_or_else(|| ","))
    }
}
