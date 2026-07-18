class Mapper {
    static int mapper(int[] values) {
        int result = 0;
        for (int value : values) {
            result = transform(value);
        }
        return result;
    }
}
