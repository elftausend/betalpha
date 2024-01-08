pub fn base36_to_base10(input: i8) -> i32 {
    let mut result = 0;
    let mut base = 1;
    let mut num = input.abs() as i32;

    while num > 0 {
        let digit = num % 10;
        result += digit * base;
        num /= 10;
        base *= 36;
    }

    result * if input.is_negative() { -1 } else { 1 }
}

pub fn look_to_i8_range(yaw: f32, pitch: f32) -> (i8, i8) {
    let yawf = ((yaw / 360.) * 255.) % 255.;
    let pitch = (((pitch / 360.) * 255.) % 255.) as i8;

    let mut yaw = yawf as i8;
    if yawf < -128. {
        yaw = 127 - (yawf + 128.).abs() as i8
    }
    if yawf > 128. {
        yaw = -128 + (yawf - 128.).abs() as i8
    }
    (yaw, pitch)
}

#[test]
fn test_base_conv() {
    assert_eq!(base36_to_base10(18), 44);
    println!("base: {}", base36_to_base10(-127));
    println!("{}", (12 << 4));
}
