use std::time::Duration;

use crate::handles::{SqlResult, Statement, StatementRef};

pub trait Cancelable {
    fn cancelled(&self) -> bool;
}

fn run_or_cancel_impl<F, O>(
    mut f: F,
    sleep: Duration,
    cancelable: &impl Cancelable,
) -> SqlResult<Option<O>>
where
    F: FnMut(bool) -> SqlResult<Option<O>>,
{
    let mut ret = (f)(cancelable.cancelled());

    // Wait for operation to finish, using polling method
    while matches!(ret, SqlResult::StillExecuting) {
        std::thread::sleep(sleep);
        ret = (f)(cancelable.cancelled());
    }
    ret
}

pub fn run_or_cancel<F, O>(mut f: F, sleep: Duration, cancelable: &impl Cancelable) -> SqlResult<O>
where
    F: FnMut(bool) -> SqlResult<O>,
{
    let mut ret = (f)(cancelable.cancelled());

    // Wait for operation to finish, using polling method
    while matches!(ret, SqlResult::StillExecuting) {
        std::thread::sleep(sleep);
        ret = (f)(cancelable.cancelled());
    }
    ret
}

pub fn run_or_cancel_stmt<F, O, S>(
    statement: &mut S,
    mut f: F,
    sleep: Duration,
    cancelable: &impl Cancelable,
) -> SqlResult<Option<O>>
where
    F: FnMut(&mut S) -> SqlResult<O>,
    S: Statement,
{
    run_or_cancel_impl(
        |cancelled: bool| {
            let res = f(statement);
            if matches!(res, SqlResult::StillExecuting) && cancelled {
                statement.cancel().map(|_| None)
            } else {
                res.map(Some)
            }
        },
        sleep,
        cancelable,
    )
}
