class Filter {
    static int filter_positive(int[] values) {
        int count = 0;
        for (int value : values) {
            if (value > 0) {
                count += 1;
            }
        }
        return count;
    }
}
