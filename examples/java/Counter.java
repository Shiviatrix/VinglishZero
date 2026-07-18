class Counter {
    static int count_items(int[] items) {
        int count = 0;
        for (int item : items) {
            count += 1;
        }
        return count;
    }
}
