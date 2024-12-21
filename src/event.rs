pub enum Event {
    Enter(u32),

    WaitingForWorker(u32, u32, bool),
    // LeavePlace,
    ConsumeFood(u32),
    WorkerWalkingDance(u32),
    WaitingForFood(u32), //random val
}
