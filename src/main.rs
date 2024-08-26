use bollard::container::{ListContainersOptions, StatsOptions};
use bollard::Docker;

use futures_util::stream::StreamExt;

use std::collections::HashMap;

#[derive(Debug)]
struct PortholeCpuStats {
    per_cpu_usaage: Option<Vec<u64>>,
    usage_in_usermode: u64,
    total_usage: u64,
    usage_in_kernelmode: u64,
    system_cpu_usage: Option<u64>,
    online_cpus: Option<u64>,
}

#[derive(Debug)]
struct PortholeMemoryStats {
    max_usage: Option<u64>,
    usage: Option<u64>,
    limit: Option<u64>,
}

struct PortholeStats {
    cpu: PortholeCpuStats,
    memory: PortholeMemoryStats,
}

struct Container {
    id: String,
    names: Option<Vec<String>>,
    image: Option<String>,
    stats: PortholeStats,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    let mut filter = HashMap::new();
    filter.insert(String::from("status"), vec![String::from("running")]);
    let containers = &docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: filter,
            ..Default::default()
        }))
        .await?;

    if containers.is_empty() {
        panic!("No running containers");
    } else {
        for container in containers {
            let container_id = container.id.as_ref().unwrap();
            let stream = &mut docker
                .stats(
                    container_id,
                    Some(StatsOptions {
                        stream: false,
                        ..Default::default()
                    }),
                )
                .take(1);

            while let Some(Ok(stats)) = stream.next().await {
                let cpu = PortholeCpuStats {
                    per_cpu_usaage: stats.cpu_stats.cpu_usage.percpu_usage,
                    usage_in_usermode: stats.cpu_stats.cpu_usage.usage_in_usermode,
                    total_usage: stats.cpu_stats.cpu_usage.total_usage,
                    usage_in_kernelmode: stats.cpu_stats.cpu_usage.usage_in_kernelmode,
                    system_cpu_usage: stats.cpu_stats.system_cpu_usage,
                    online_cpus: stats.cpu_stats.online_cpus,
                };

                let porthole_stats = PortholeStats {
                    cpu: cpu,
                    memory: PortholeMemoryStats {
                        max_usage: stats.memory_stats.max_usage,
                        usage: stats.memory_stats.usage,
                        limit: stats.memory_stats.limit,
                    },
                };

                let porthole_container = Container {
                    id: container_id.clone(),
                    names: container.names.clone(),
                    image: container.image.clone(),
                    stats: porthole_stats,
                };

                println!("Container ID: {:?}", porthole_container.id);
                println!("Names: {:?}", porthole_container.names);
                println!("Image: {:?}", porthole_container.image);
                println!("CPU: {:?}", porthole_container.stats.cpu);
                println!("Memory: {:?}", porthole_container.stats.memory);
            }
        }
    }

    Ok(())
}
