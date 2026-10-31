use crate::{Node, home::Delivery, model::now};
use std::time::Duration;

impl Node {
    pub(crate) fn start_deliveries(&self) {
        let weak = std::sync::Arc::downgrade(&self.0);
        tokio::spawn(async move {
            if let Some(inner) = weak.upgrade() {
                for mut job in inner.store.all::<Delivery>("delivery").unwrap_or_default() {
                    if job.state == "sending" {
                        job.state = "waiting".into();
                        job.next_attempt = 0;
                        let _ = inner.store.put("delivery", &job.id, &job);
                    }
                }
            }
            loop {
                let Some(inner) = weak.upgrade() else { break };
                tokio::select! {_=inner.shutdown.cancelled()=>break,_=inner.delivery_wake.notified()=>{},_=tokio::time::sleep(Duration::from_secs(5))=>{}}
                let node = Node(inner);
                let mut jobs = node.0.store.all::<Delivery>("delivery").unwrap_or_default();
                jobs.sort_by_key(|j| j.created_at);
                let Some(mut job) = jobs
                    .into_iter()
                    .find(|j| j.state == "waiting" && j.next_attempt <= now())
                else {
                    continue;
                };
                job.state = "sending".into();
                job.message = format!("Sending to {}", job.owner.name);
                job.updated_at = now();
                if node.0.store.put("delivery", &job.id, &job).is_err() {
                    continue;
                }
                let result =
                    tokio::select! {_=node.0.shutdown.cancelled()=>break,r=node.deliver(&job)=>r};
                job.attempts = job.attempts.saturating_add(1);
                job.updated_at = now();
                match result {
                    Ok(Ok(receipt)) => {
                        job.state = "delivered".into();
                        job.message = receipt.message;
                    }
                    Ok(Err(message)) => {
                        job.state = "failed".into();
                        job.message = message;
                    }
                    Err(_) => {
                        job.state = "waiting".into();
                        job.message = format!(
                            "Waiting for {}. Will retry automatically while Hopper is open.",
                            job.owner.name
                        );
                        job.next_attempt = now() + u64::from(job.attempts.min(10)) * 30;
                    }
                }
                let _ = node.0.store.put("delivery", &job.id, &job);
                if job.state == "delivered" {
                    let _ = node.refresh().await;
                }
                node.0.delivery_wake.notify_one();
            }
        });
    }
}
