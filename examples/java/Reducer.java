class Reducer {
    static int reducer(int[] values) {
        int result = 0;
        for (int value : values) {
            result = combine(result, value);
        }
        return result;
    }
}
