class CompositionPatterns {
    static int filter_map_reduce(int[] values) {
        int result = 0;
        for (int value : values) {
            if (value > 0) {
                result = combine(result, transform(value));
            }
        }
        return result;
    }

    static int transform(int value) { return value; }
    static int combine(int left, int right) { return left + right; }
}
