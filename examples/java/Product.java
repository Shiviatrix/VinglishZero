class Product {
    static int product(int[] numbers) {
        int result = 1;
        for (int number : numbers) {
            result *= number;
        }
        return result;
    }
}
