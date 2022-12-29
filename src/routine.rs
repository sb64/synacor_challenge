use std::collections::HashMap;

use rayon::{
    prelude::{IntoParallelIterator, ParallelIterator},
    ThreadPoolBuilder,
};

type Regs = (u16, u16);

struct Search {
    r7: u16,
    memo: HashMap<Regs, Regs>,
}

impl Search {
    fn new(r7: u16) -> Self {
        Self {
            r7,
            memo: HashMap::new(),
        }
    }

    fn find(&mut self, regs: Regs) -> Regs {
        if let Some(&ret) = self.memo.get(&regs) {
            return ret;
        }

        if regs.0 == 0 {
            let ret = ((regs.1 + 1) & 0x7fff, regs.1);
            self.memo.insert(regs, ret);
            return ret;
        }

        if regs.1 == 0 {
            let ret = self.find((regs.0 - 1, self.r7));
            self.memo.insert(regs, ret);
            return ret;
        }

        let ret = self.find((regs.0, regs.1 - 1));
        let ret = self.find((regs.0 - 1, ret.0));
        self.memo.insert(regs, ret);
        ret
    }
}

#[test]
fn find_magic_value() {
    ThreadPoolBuilder::new()
        .stack_size(24 * 1024 * 1024)
        .build_global()
        .unwrap();

    let magic_number = (1..(1 << 15))
        .into_par_iter()
        .filter_map(|r7| {
            if r7 % 1000 == 0 {
                println!("working on r7 = {r7}");
            }
            let mut search = Search::new(r7);
            let result = search.find((4, 1));
            if result.0 == 6 {
                Some(r7)
            } else {
                None
            }
        })
        .find_any(|_| true)
        .unwrap();
    println!("{magic_number}");
}
