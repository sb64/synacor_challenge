use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Square {
    Num(i32),
    Add,
    Sub,
    Mult,
}

const GRID: [[Square; 4]; 4] = [
    [Square::Mult, Square::Num(8), Square::Sub, Square::Num(1)],
    [Square::Num(4), Square::Mult, Square::Num(11), Square::Mult],
    [Square::Add, Square::Num(4), Square::Sub, Square::Num(18)],
    [Square::Num(22), Square::Sub, Square::Num(9), Square::Mult],
];

#[test]
fn traverse_grid() {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::from([(0, 3, 0, Some(Square::Add), Vec::new())]);
    while let Some((x, y, weight, op, path)) = queue.pop_front() {
        if visited.contains(&(x, y, weight, op)) {
            continue;
        }

        visited.insert((x, y, weight, op));

        let (new_weight, new_op) = match (GRID[y][x], op) {
            (Square::Num(num), Some(Square::Add)) => (weight + num, None),
            (Square::Num(num), Some(Square::Sub)) => (weight - num, None),
            (Square::Num(num), Some(Square::Mult)) => (weight * num, None),
            (op @ Square::Add | op @ Square::Sub | op @ Square::Mult, None) => (weight, Some(op)),
            (square @ Square::Num(_), None) => panic!("there's a square with a number but no preceding op: x = {}, y = {}, square = {:?}", x, y, square),
            (square @ Square::Add | square @ Square::Sub | square @ Square::Mult, Some(op)) => panic!("there's a square with an op but also a preceding op: x = {}, y = {}, square = {:?}, op = {:?}", x, y, square, op),
            (square @ Square::Num(_), Some(before_square @ Square::Num(_))) => panic!("there's a square with a number before it: x = {}, y = {}, square = {:?}, previous square = {:?}", x, y, square, before_square),
        };

        if x == 3 && y == 0 {
            if new_weight == 30 {
                println!("the path is: {path:?}");
                return;
            } else {
                continue;
            }
        }

        if x > 0 && y != 3 && !visited.contains(&(x - 1, y, new_weight, new_op)) {
            let mut new_path = path.clone();
            new_path.push("left");
            queue.push_back((x - 1, y, new_weight, new_op, new_path));
        }

        if x < 3 && !visited.contains(&(x + 1, y, new_weight, new_op)) {
            let mut new_path = path.clone();
            new_path.push("right");
            queue.push_back((x + 1, y, new_weight, new_op, new_path));
        }

        if y > 0 && !visited.contains(&(x, y - 1, new_weight, new_op)) {
            let mut new_path = path.clone();
            new_path.push("up");
            queue.push_back((x, y - 1, new_weight, new_op, new_path));
        }

        if y < 3 && x != 0 && !visited.contains(&(x, y + 1, new_weight, new_op)) {
            let mut new_path = path;
            new_path.push("down");
            queue.push_back((x, y + 1, new_weight, new_op, new_path));
        }
    }
}
