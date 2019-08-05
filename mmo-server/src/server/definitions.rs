//Server requests
    pub enum ServerRoute{
        Time=0,
        Register=1,
        Ack=2,
        CallbackUpdate=3,
        Rpc=4,
        PropertyRep=5,
        FunctionRep=6,
        Authority=7,
        State=8,
        Joined=9,
        Closed=10,
        SessionUpdate=11,
        PlayerStateUpdate=12,
        RegistrationSpawned=13,
        RegisterProperty=14,
        RequestUserState=15,
        RequestObjectState=16,
        RequestPropertyState=17,
        Rejoin=18,
        UnregisterObject=19,
        Voice=20,
    }
    pub enum ClientRoute{
        Ping=0,
        World=1,
        Avatar=2,
        ByteArray=4,
    }
    pub enum Relevancy{
        InitialOnly=0, // - This property will only attempt to send on the initial bunch
        OwnerOnly=1, // - This property will only send to the actor's owner
        SkipOwner=2, // - This property send to every connection EXCEPT the owner
        SimulatedOnly=3, // - This property will only send to simulated actors
        AutonomousOnly=4, // - This property will only send to autonomous actors
        SimulatedOrPhysics=5, //- This property will send to simulated OR bRepPhysics actors
        InitialOrOwner=6, // - This property will send on the initial packet, or to the actors owner
        Custom=7, // - This property has no particular condition, but wants the ability to toggle on/off via SetCustomIsActiveOverride
        None=8, // - This property will send to sender, and all listeners
        Skip=9,
   }