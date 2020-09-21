
pub enum CursorIndex {
    LeftPointer = 0,
}

#[repr(u16)]
#[derive(Copy, Clone)]
pub enum CoreCursor {
    /*
    XCursor = 0,
    Arrow = 1,
    BasedArrowDown = 2,
    BasedArrowUp = 3,
    Boat = 4,
    Bogosity = 5,
    BottomLeftCorner = 6,
    BottomRightCorner = 7,
    BottomSide = 8,
    BottomTee = 9,
    BoxSpiral = 10,
    CenterPtr = 11,
    Circle = 12,
    Clock = 13,
    CoffeeMug = 14,
    Cross = 15,
    CrossReverse = 16,
    Crosshair = 17,
    DiamondCross = 18,
    Dot = 19,
    Dotbox = 20,
    DoubleArrow = 21,
    DraftLarge = 22,
    DraftSmall = 23,
    DrapedBox = 24,
    Exchange = 25,
    Fleur = 26,
    Gobbler = 27,
    Gumby = 28,
    Hand1 = 29,
    Hand2 = 30,
    Heart = 31,
    Icon = 32,
    IronCross = 33,
    */
    LeftPtr = 34,
    /*
    LeftSide = 35,
    LeftTee = 36,
    LeftButton = 37,
    LlAngle = 38,
    LrAngle = 39,
    Man = 40,
    MiddleButton = 41,
    Mouse = 42,
    Pencil = 43,
    Pirate = 44,
    Plus = 45,
    QuestionArrow = 46,
    RightPtr = 47,
    RightSide = 48,
    RightTee = 49,
    RightButton = 50,
    RtlLogo = 51,
    Sailboat = 52,
    SbDownArrow = 53,
    SbHDoubleArrow = 54,
    SbLeftArrow = 55,
    SbRightArrow = 56,
    SbUpArrow = 57,
    SbVDoubleArrow = 58,
    Shuttle = 59,
    Sizing = 60,
    Spider = 61,
    Spraycan = 62,
    Star = 63,
    Target = 64,
    Tcross = 65,
    TopLeftArrow = 66,
    TopLeftCorner = 67,
    TopRightArrow = 68,
    TopSide = 69,
    TopTee = 70,
    Trek = 71,
    UlAngle = 72,
    Umbrella = 73,
    UrAngle = 74,
    Watch = 75,
    Xterm = 76,
    */
}

impl CoreCursor {
    pub fn value(self) -> u16 {
        return (self as u16) * 2;
    }

    pub fn mask(self) -> u16 {
        return self.value() + 1;
    }
}