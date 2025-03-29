use plotters::prelude::*;
use std::thread;
use std::time::Duration;

pub trait RateLimiter {
    // weight should typically be 1, but we make this configurable as it is possible that not all
    // requests are created equal
    //
    // e.g if we have two request types A, B where A => low memory, CPU consumption GET request and
    // B => high CPU/Memory consumption request, then we may want to assign triple the weight to B
    // to ensure that a client cannot run too many of request B in a short period
    fn is_rate_limited(&self, weight: u64) -> bool;

    // Perform any cleanup necessary such as dropping running background threads
    fn cleanup(&self) {}
}

pub fn run_simulation<T: RateLimiter>(
    rate_limiter: &T,
    requests_per_second: &[u64],
    capacity: u64,
    graph_title: &str,
) {
    let mut allowed = Vec::new();
    let mut denied = Vec::new();

    for request_rate in requests_per_second {
        let mut allowed_count = 0;
        let mut denied_count = 0;

        for _ in 0..*request_rate {
            if rate_limiter.is_rate_limited(1) {
                allowed_count += 1;
            } else {
                denied_count += 1;
            }
            thread::sleep(Duration::from_millis(1000 / request_rate));
        }

        allowed.push(allowed_count);
        denied.push(denied_count);
    }

    rate_limiter.cleanup();
    draw_graph(&allowed, &denied, capacity, graph_title).unwrap();
}

fn draw_graph(
    allowed: &[u64],
    denied: &[u64],
    capacity: u64,
    graph: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(&graph, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let max_y = capacity as u32;

    let mut chart = ChartBuilder::on(&root)
        .caption(graph, ("sans-serif", 40))
        .margin(10)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(0..allowed.len() as u32, 0..max_y)?;

    chart.configure_mesh().draw()?;

    chart
        .draw_series(LineSeries::new(
            allowed
                .iter()
                .enumerate()
                .map(|(x, &y)| (x as u32, y as u32)),
            &BLUE,
        ))?
        .label("Allowed")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart
        .draw_series(LineSeries::new(
            denied
                .iter()
                .enumerate()
                .map(|(x, &y)| (x as u32, y as u32)),
            &RED,
        ))?
        .label("Denied")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart
        .configure_series_labels()
        .background_style(&WHITE)
        .draw()?;

    println!("Graph saved to {graph}");

    Ok(())
}
