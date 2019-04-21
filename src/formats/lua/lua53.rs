// Not sure about that
struct Header {
    format:        u8,
    conv_data:     [u8; 6],
    int_sz:        u8,
    size_t_sz:     u8,
    intr_sz:       u8,
    lua_number_sz: u8,
    float_sz:      u8,
}
