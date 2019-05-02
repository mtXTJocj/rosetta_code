fn main() {
    let mut doors = [false; 100];

    for stride in 1..101 {
        let mut idx = stride;
        while idx <= 100 {
            doors[idx - 1] = !doors[idx - 1];
            idx += stride;
        }
    }

    for (i, &is_opend) in doors.iter().enumerate() {
        println!(
            "Door {} is {}",
            i + 1,
            if is_opend { "opened" } else { "closed" }
        );
    }
}
