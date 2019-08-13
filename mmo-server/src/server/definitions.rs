//Server requests
use std::intrinsics::transmute;
    #[derive(PartialEq,Debug)]
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
impl From<u8> for ServerRoute {
    fn from(t:u8) -> ServerRoute {
        assert!(ServerRoute::Time as u8 <= t && t <= ServerRoute::Voice as u8);
        unsafe { transmute(t) }
    }
}
    #[derive(PartialEq,Debug)]
    pub enum ClientRoute{
        Ping=0,
        World=1,
        Avatar=2,
        ByteArray=4,
    }
impl From<u8> for ClientRoute {
    fn from(t:u8) -> ClientRoute {
        assert!(ClientRoute::Ping as u8 <= t && t <= ClientRoute::ByteArray as u8);
        unsafe { transmute(t) }
    }
}
    #[derive(PartialEq)]
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