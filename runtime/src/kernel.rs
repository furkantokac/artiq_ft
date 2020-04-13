use libcortex_a9::sync_channel;

pub fn main(mut sc_tx: sync_channel::Sender<usize>, mut sc_rx: sync_channel::Receiver<usize>) {
    for i in sc_rx {
        sc_tx.send(*i * *i);
    }

	loop {}
}
