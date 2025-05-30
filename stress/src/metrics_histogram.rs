/*
    Stress test results:
    OS: Ubuntu 22.04.4 LTS (5.15.153.1-microsoft-standard-WSL2)
    Hardware: Intel(R) Xeon(R) Platinum 8370C CPU @ 2.80GHz, 16vCPUs,
    RAM: 64.0 GB
    ~9.0 M/sec

    Hardware: AMD EPYC 7763 64-Core Processor - 2.44 GHz, 16vCPUs,
    ~12.0 M /sec
*/

use lazy_static::lazy_static;
use opentelemetry::{
    metrics::{Histogram, MeterProvider as _},
    KeyValue,
};
use opentelemetry_sdk::metrics::{ManualReader, SdkMeterProvider};
use rand::{
    rngs::{self},
    Rng, SeedableRng,
};
use std::cell::RefCell;

mod throughput;

lazy_static! {
    static ref PROVIDER: SdkMeterProvider = SdkMeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build();
    static ref ATTRIBUTE_VALUES: [&'static str; 10] = [
        "value1", "value2", "value3", "value4", "value5", "value6", "value7", "value8", "value9",
        "value10"
    ];
    static ref HISTOGRAM: Histogram<u64> = PROVIDER.meter("test").u64_histogram("hello").build();
}

thread_local! {
    /// Store random number generator for each thread
    static CURRENT_RNG: RefCell<rngs::SmallRng> = RefCell::new(rngs::SmallRng::from_os_rng());

    static PROVIDER_PER_THREAD: SdkMeterProvider = SdkMeterProvider::builder()
        .with_reader(ManualReader::builder().build())
        .build();

    static HISTOGRAM_PER_THREAD: Histogram<u64> = PROVIDER_PER_THREAD.with(|h|h.meter("test").u64_histogram("hello").build());
}

fn main() {
    match std::env::args().find(|arg| arg == "--per-thread") {
        None => throughput::test_throughput(test_histogram_shared),
        Some(_) => throughput::test_throughput(test_histogram_per_thread),
    }
}

fn test_histogram_shared() {
    test_histogram(&HISTOGRAM);
}

fn test_histogram_per_thread() {
    HISTOGRAM_PER_THREAD.with(test_histogram);
}

fn test_histogram(histogram: &Histogram<u64>) {
    let len = ATTRIBUTE_VALUES.len();
    let rands = CURRENT_RNG.with(|rng| {
        let mut rng = rng.borrow_mut();
        [
            rng.random_range(0..len),
            rng.random_range(0..len),
            rng.random_range(0..len),
        ]
    });
    let index_first_attribute = rands[0];
    let index_second_attribute = rands[1];
    let index_third_attribute = rands[2];

    // each attribute has 10 possible values, so there are 1000 possible combinations (time-series)
    histogram.record(
        1,
        &[
            KeyValue::new("attribute1", ATTRIBUTE_VALUES[index_first_attribute]),
            KeyValue::new("attribute2", ATTRIBUTE_VALUES[index_second_attribute]),
            KeyValue::new("attribute3", ATTRIBUTE_VALUES[index_third_attribute]),
        ],
    );
}
