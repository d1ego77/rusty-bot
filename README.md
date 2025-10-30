# rusty-bot
Rust software for the Flysky-i6x radio control and Arduino Robot of Mecanum wheels

Software;
rustup toolchain install nightly

**On Mac OS**
```
xcode-select --install # for the fist time

brew tap osx-cross/avr

brew install avr-binutils avr-gcc avrdude

cargo +stable install ravedude
```

**Parts:**

- Flysky-i6x
- Arduino Uno R3
- Motor Driver Adafruit TB661 (x2)
- 7.2V 2400mAh NiMH Battery 
- Mecanum Wheel Chassis Car Kit with TT Motor
