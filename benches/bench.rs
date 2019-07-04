#![feature(test, async_await, await_macro)]
#![warn(rust_2018_idioms)]

extern crate test;

const TASKS: usize = 300;
const STEPS: usize = 300;
const LIGHT_TASKS: usize = 25_000;

macro_rules! benchmark_legacy {
    (smoke, $run:path, $spawn:path) => {
        #[bench]
        fn smoke(b: &mut test::Bencher) {
            use crossbeam_utils::sync::WaitGroup;

            b.iter(move || {
                let wg = WaitGroup::new();
                $run({
                    let wg = wg.clone();
                    future::lazy(move || {
                        $spawn(future::lazy(move || {
                            drop(wg);
                            Ok(())
                        }));
                        Ok(())
                    })
                });
                wg.wait();
            });
        }
    };

    (notify_self, $run:path, $spawn:path) => {
        #[bench]
        fn notify_self(b: &mut test::Bencher) {
            use crate::{STEPS, TASKS};
            use crossbeam_utils::sync::WaitGroup;
            use futures::{future, task, Async};

            b.iter(move || {
                let wg = WaitGroup::new();
                $run({
                    let wg = wg.clone();
                    future::lazy(move || {
                        for _ in 0..TASKS {
                            let mut depth = 0;
                            let wg = wg.clone();

                            $spawn(future::poll_fn(move || {
                                let _capture = &wg;
                                depth += 1;

                                if depth == STEPS {
                                    Ok(Async::Ready(()))
                                } else {
                                    task::current().notify();
                                    Ok(Async::NotReady)
                                }
                            }));
                        }
                        drop(wg);
                        Ok(())
                    })
                });
                wg.wait();
            });
        }
    };

    (spawn_many, $run:path, $spawn:path) => {
        #[bench]
        fn spawn_many(b: &mut test::Bencher) {
            use crate::LIGHT_TASKS;
            use crossbeam_utils::sync::WaitGroup;
            use futures::future;

            b.iter(move || {
                let wg = WaitGroup::new();
                $run({
                    let wg = wg.clone();
                    future::lazy(move || {
                        for _ in 0..LIGHT_TASKS {
                            let wg = wg.clone();
                            $spawn(future::lazy(move || {
                                drop(wg);
                                Ok(())
                            }));
                        }
                        Ok(())
                    })
                });
                wg.wait();
            });
        }
    };

    (poll_reactor, $run:path, $spawn:path) => {
        use crate::{STEPS, TASKS};
        use crossbeam_utils::sync::WaitGroup;
        use futures::{future, Async};
        use tokio::reactor::Registration;

        #[bench]
        fn poll_reactor(b: &mut test::Bencher) {
            b.iter(|| {
                let wg = WaitGroup::new();
                $run({
                    let wg = wg.clone();
                    future::lazy(move || {
                        for _ in 0..TASKS {
                            let wg = wg.clone();
                            $spawn(future::lazy(move || {
                                let (r, s) = mio::Registration::new2();
                                let registration = Registration::new();
                                registration.register(&r).unwrap();

                                let mut depth = 0;

                                $spawn(future::poll_fn(move || {
                                    let _capture = (&wg, &r);
                                    loop {
                                        if registration.poll_read_ready().unwrap().is_ready() {
                                            depth += 1;
                                            if depth == STEPS {
                                                return Ok(Async::Ready(()));
                                            }
                                        } else {
                                            s.set_readiness(mio::Ready::readable()).unwrap();
                                            return Ok(Async::NotReady);
                                        }
                                    }
                                }));
                                Ok(())
                            }));
                        }
                        Ok(())
                    })
                });
                wg.wait();
            })
        }
    };
}

macro_rules! benchmark_preview {
    (smoke, $run:path, $spawn:path) => {
        #[bench]
        fn smoke(b: &mut test::Bencher) {
            use crossbeam_utils::sync::WaitGroup;

            b.iter(move || {
                let wg = WaitGroup::new();
                $run({
                    let wg = wg.clone();
                    async move {
                        $spawn(async move {
                            drop(wg);
                        });
                    }
                });
                wg.wait();
            });
        }
    };

    (notify_self, $run:path, $spawn:path) => {
        #[bench]
        fn notify_self(b: &mut test::Bencher) {
            use crate::{TASKS, STEPS};
            use crossbeam_utils::sync::WaitGroup;
            use futures_preview::future::Future;
            use futures_preview::task::{Poll, Context};
            use std::pin::Pin;

            struct Task {
                depth: usize,
            }

            impl Future for Task {
                type Output = ();

                fn poll(mut self: Pin<&mut Self>, w: &mut Context<'_>) -> Poll<Self::Output> {
                    self.depth += 1;

                    if self.depth == STEPS {
                        Poll::Ready(())
                    } else {
                        w.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }

            b.iter(move || {
                let wg = WaitGroup::new();
                $run({
                    let wg = wg.clone();
                    async move {
                        for _ in 0..TASKS {
                            let wg = wg.clone();
                            $spawn(async move {
                                await!(Task { depth: 0 });
                                drop(wg);
                            });
                        }
                    }
                });
                wg.wait();
            });
        }
    };

    (spawn_many, $run:path, $spawn:path) => {
        #[bench]
        fn spawn_many(b: &mut test::Bencher) {
            use crate::LIGHT_TASKS;
            use crossbeam_utils::sync::WaitGroup;

            b.iter(move || {
                let wg = WaitGroup::new();
                $run({
                    let wg = wg.clone();
                    async move {
                        for _ in 0..LIGHT_TASKS {
                            let wg = wg.clone();
                            $spawn(async move {
                                drop(wg);
                            });
                        }
                    }
                });
                wg.wait();
            });
        }
    };

    (poll_reactor, $run:path, $spawn:path) => {
        use crate::{TASKS, STEPS};
        use crossbeam_utils::sync::WaitGroup;
        use futures::{future, Async};
        use futures_util::compat::Future01CompatExt;
        use futures_util::future::FutureExt;
        use tokio::reactor::Registration;

        #[bench]
        fn poll_reactor(b: &mut test::Bencher) {
            b.iter(|| {
                let wg = WaitGroup::new();
                $run({
                    let wg = wg.clone();
                    async move {
                        for _ in 0..TASKS {
                            let wg = wg.clone();
                            $spawn(async move {
                                let (r, s) = mio::Registration::new2();
                                let registration = Registration::new();
                                registration.register(&r).unwrap();

                                let mut depth = 0;
                                let mut capture = Some((wg, r));

                                $spawn(
                                    future::poll_fn(move || {
                                        loop {
                                            if registration.poll_read_ready().unwrap().is_ready() {
                                                depth += 1;
                                                if depth == STEPS {
                                                    capture.take().unwrap();
                                                    return Ok(Async::Ready(()));
                                                }
                                            } else {
                                                s.set_readiness(mio::Ready::readable()).unwrap();
                                                return Ok(Async::NotReady);
                                            }
                                        }
                                    })
                                    .compat()
                                    .map(|_: Result<_, ()>| ())
                                );
                            });
                        }
                    }
                });
                wg.wait();
            });
        }
    };
}

mod tokio {
    use tokio::{run, spawn};

    benchmark_legacy!(smoke, run, spawn);
    benchmark_legacy!(notify_self, run, spawn);
    benchmark_legacy!(spawn_many, run, spawn);
    benchmark_legacy!(poll_reactor, run, spawn);
}

mod tokio_current_thread {
    use tokio::runtime::current_thread::{run, spawn};

    benchmark_legacy!(smoke, run, spawn);
    benchmark_legacy!(notify_self, run, spawn);
    benchmark_legacy!(spawn_many, run, spawn);
    benchmark_legacy!(poll_reactor, run, spawn);
}

mod tokio_io_pool {
    use tokio::spawn;
    use tokio_io_pool::run;

    benchmark_legacy!(smoke, run, spawn);
    benchmark_legacy!(notify_self, run, spawn);
    benchmark_legacy!(spawn_many, run, spawn);
    benchmark_legacy!(poll_reactor, run, spawn);
}

mod juliex {
    use juliex::spawn;

    benchmark_preview!(smoke, spawn, spawn);
    benchmark_preview!(notify_self, spawn, spawn);
    benchmark_preview!(spawn_many, spawn, spawn);
    benchmark_preview!(poll_reactor, spawn, spawn);
}
