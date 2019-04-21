struct Header {
    format:        u8,
    endianness:    u8,
    int_sz:        u8,
    size_t_sz:     u8,
    intr_sz:       u8,
    lua_number_sz: u8,
    integral_flag: u8,
}
