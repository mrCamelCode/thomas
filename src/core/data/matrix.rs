use super::{Dimensions2d, IntCoords2d};

/// A 2D matrix with defined iteration order.
pub struct Matrix<T> {
    matrix: Vec<Vec<MatrixCell<T>>>,
    dimensions: Dimensions2d,
}
impl<T> Matrix<T> {
    pub fn new<F>(dimensions: Dimensions2d, default_cell_generator: F) -> Self
    where
        F: Fn() -> T,
    {
        let mut matrix = vec![];

        for row in 0..dimensions.height() {
            matrix.push(vec![]);

            for column in 0..dimensions.width() {
                matrix[row as usize].push(MatrixCell::new(
                    IntCoords2d::new(column as i64, row as i64),
                    default_cell_generator(),
                ));
            }
        }

        Self { matrix, dimensions }
    }

    pub fn get(&self, x: u64, y: u64) -> Option<&MatrixCell<T>> {
        if x < self.dimensions.width() && y < self.dimensions.height() {
            return Some(&self.matrix[y as usize][x as usize]);
        }

        None
    }

    pub fn update_cell_at(&mut self, x: u64, y: u64, data: T) {
        if x < self.dimensions.width() && y < self.dimensions.height() {
            let mut cell = &mut self.matrix[y as usize][x as usize];

            cell.data = data;
        }
    }

    /// Produces an interator that iterates over the cells in the Matrix from
    /// the top left to bottom right, scanning to the end of a row before going
    /// to the next.
    pub fn iter(&self) -> MatrixIter<T> {
        MatrixIter::new(&self.matrix)
    }
}

pub struct MatrixCell<T> {
    location: IntCoords2d,
    data: T,
}
impl<T> MatrixCell<T> {
    fn new(location: IntCoords2d, data: T) -> Self {
        Self { location, data }
    }

    pub fn location(&self) -> &IntCoords2d {
        &self.location
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

pub struct MatrixIter<'a, T> {
    matrix: &'a Vec<Vec<MatrixCell<T>>>,
    curr_x: usize,
    curr_y: usize,
}
impl<'a, T> MatrixIter<'a, T> {
    fn new(matrix: &'a Vec<Vec<MatrixCell<T>>>) -> Self {
        Self {
            matrix,
            curr_x: 0,
            curr_y: 0,
        }
    }
}
impl<'a, T> Iterator for MatrixIter<'a, T> {
    type Item = &'a MatrixCell<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(row) = self.matrix.get(self.curr_x) {
            if let Some(cell) = row.get(self.curr_y) {
                if self.curr_y + 1 >= row.len() {
                    self.curr_x += 1;
                    self.curr_y = 0;
                } else {
                    self.curr_y += 1;
                }

                return Some(cell);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod iter {
        use super::*;

        #[test]
        fn it_iterates_from_top_left_to_bottom_right() {
            let matrix = Matrix::new(Dimensions2d::new(3, 4), || 5);

            let mut result = String::new();

            matrix.iter().for_each(|cell| {
                result += format!("{:?}\n", cell.location.values()).as_str();
            });

            let expected = String::from(
                "(0, 0)
(1, 0)
(2, 0)
(3, 0)
(0, 1)
(1, 1)
(2, 1)
(3, 1)
(0, 2)
(1, 2)
(2, 2)
(3, 2)
",
            );

            assert_eq!(result, expected);
        }
    }

    mod update_cell_at {
        use super::*;

        #[test]
        fn it_updates_the_cell_with_provided_info() {
            let mut matrix = Matrix::new(Dimensions2d::new(3, 3), || 5);

            matrix.update_cell_at(1, 2, 9);

            assert_eq!(matrix.get(1, 2).unwrap().data, 9);
        }

        #[test]
        fn it_does_nothing_when_given_a_bad_coord() {
            let mut matrix = Matrix::new(Dimensions2d::new(3, 3), || 5);

            let original_data = matrix
                .iter()
                .map(|cell| cell.data.to_string())
                .collect::<Vec<String>>()
                .join(", ");

            matrix.update_cell_at(100, 2, 9);
            matrix.update_cell_at(1, 200, 10);
            matrix.update_cell_at(111, 222, 2);

            let new_data = matrix
                .iter()
                .map(|cell| cell.data.to_string())
                .collect::<Vec<String>>()
                .join(", ");

            assert_eq!(original_data, new_data);
        }
    }
}
