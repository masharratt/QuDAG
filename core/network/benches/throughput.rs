use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qudag_network::{Message, MessageHandler, PeerId, Route};
use tokio::runtime::Runtime;

fn benchmark_message_throughput(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("message_throughput_10k", |b| {
        b.iter(|| {
            rt.block_on(async {
                let handler = MessageHandler::new();
                let mut handles = vec![];
                
                // Spawn 4 sender tasks
                for i in 0..4 {
                    let handler = handler.clone();
                    let handle = tokio::spawn(async move {
                        for j in 0..2500 { // 2500 * 4 = 10k messages
                            let msg = Message::new(
                                format!("bench_msg_{}_{}", i, j).into(),
                                PeerId::random(),
                                Route::direct(),
                            );
                            black_box(handler.send(msg).await.unwrap());
                        }
                    });
                    handles.push(handle);
                }
                
                // Wait for all sends to complete
                for handle in handles {
                    handle.await.unwrap();
                }
            })
        })
    });
    
    c.bench_function("message_routing_anonymous", |b| {
        b.iter(|| {
            rt.block_on(async {
                let handler = MessageHandler::new();
                
                let route = Route::new()
                    .add_hop(PeerId::random())
                    .add_hop(PeerId::random())
                    .add_hop(PeerId::random());
                    
                let msg = Message::new(
                    black_box(vec![0u8; 1024]), // 1KB payload
                    PeerId::random(),
                    route,
                );
                
                black_box(handler.send(msg).await.unwrap());
                black_box(handler.receive().await.unwrap());
            })
        })
    });
}

criterion_group!(benches, benchmark_message_throughput);
criterion_main!(benches);