use redis_clone::frame::parse_stream;
use redis_clone::handler::handle_request;
use redis_clone::table::Table;
use std::io::Cursor;
use std::sync::Arc;

fn main() {
    divan::main();
}

#[divan::bench(args = [1, 50, 100, 1000, 10000000])]
fn test_set_raw(bencher: divan::Bencher, amount: u32) {
    let table: Table = Table::new();

    bencher.bench_local(|| {
        for iteration in 0..amount {
            table.set(format!("key_{}", iteration), format!("val_{}", iteration));
        }
    });
}

#[divan::bench(args = [1, 50, 100, 1000, 10000000])]
fn test_get_raw(bencher: divan::Bencher, amount: u32) {
    let table: Table = Table::new();

    for iteration in 0..amount {
        table.set(format!("key_{}", iteration), format!("val_{}", iteration));
    }

    bencher.bench_local(|| {
        for iteration in 0..amount {
            table.get(&format!("key_{}", iteration));
        }
    });
}
#[divan::bench(args = [1, 50, 100, 1000, 10000000])]
fn test_set_accurate(bencher: divan::Bencher, amount: u32) {
    let table: Arc<Table> = Arc::new(Table::new());

    bencher
        .with_inputs(|| generate_bulk_commands(amount))
        .bench_local_values(|payload| {
            let mut buf_reader = Cursor::new(payload);

            // Loop until the buffer is fully consumed or parsing fails
            while (buf_reader.position() as usize) < buf_reader.get_ref().len() {
                match parse_stream(&mut buf_reader) {
                    Ok(req) => {
                        handle_request(req, &table).unwrap();
                    }
                    Err(e) => {
                        eprintln!("Error while parsing stream: {}", e);
                        break;
                    }
                }
            }
        });
}

fn generate_bulk_commands(amount: u32) -> String {
    let mut commands = String::new();

    for i in 0..amount {
        let key = format!("key{}", i);
        let val = format!("value{}", i);

        // Formats as a RESP array of 3 bulk strings: SET, key, value
        // Example output for i=0: *3\r\n$3\r\nSET\r\n$4\r\nkey0\r\n$6\r\nvalue0\r\n
        let cmd = format!(
            "*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n",
            key.len(),
            key,
            val.len(),
            val
        );

        commands.push_str(&cmd);
    }

    commands
}
