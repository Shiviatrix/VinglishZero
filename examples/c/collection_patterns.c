int sum_values(int values[], int length) { int total = 0; for (int i = 0; i < length; i += 1) { total += values[i]; } return total; }
int find_minimum(int values[], int length) { int best = values[0]; for (int i = 0; i < length; i += 1) { if (values[i] < best) { best = values[i]; } } return best; }
int count_if(int values[], int length) { int count = 0; for (int i = 0; i < length; i += 1) { if (values[i] > 0) { count += 1; } } return count; }
int any_match(int values[], int length) { for (int i = 0; i < length; i += 1) { if (values[i] > 0) { return 1; } } return 0; }
int all_positive(int values[], int length) { for (int i = 0; i < length; i += 1) { if (values[i] <= 0) { return 0; } } return 1; }
int find_first(int values[], int length, int target) { for (int i = 0; i < length; i += 1) { if (values[i] == target) { return values[i]; } } return -1; }
int find_last(int values[], int length, int target) { int last = -1; for (int i = 0; i < length; i += 1) { if (values[i] == target) { last = values[i]; } } return last; }
int reverse_traversal(int values[], int length) { int total = 0; for (int i = length - 1; i >= 0; i -= 1) { total += values[i]; } return total; }
int prefix_sum(int values[], int length) { int total = 0; for (int i = 0; i < length; i += 1) { total += values[i]; } return total; }
int frequency_counter(int values[], int length) { int count = 0; for (int i = 0; i < length; i += 1) { count += 1; } return count; }
int histogram(int values[], int length) { int count = 0; for (int i = 0; i < length; i += 1) { count += 1; } return count; }
int max_by(int values[], int length) { int best = values[0]; for (int i = 0; i < length; i += 1) { if (values[i] > best) { best = values[i]; } } return best; }
int min_by(int values[], int length) { int best = values[0]; for (int i = 0; i < length; i += 1) { if (values[i] < best) { best = values[i]; } } return best; }
int group_by(int values[], int length) { int count = 0; for (int i = 0; i < length; i += 1) { count += 1; } return count; }
int partition(int values[], int length) { int count = 0; for (int i = 0; i < length; i += 1) { if (values[i] > 0) { count += 1; } } return count; }
int unique(int values[], int length) { int count = 0; for (int i = 0; i < length; i += 1) { if (values[i] > 0) { count += 1; } } return count; }
int distinct(int values[], int length) { int count = 0; for (int i = 0; i < length; i += 1) { if (values[i] > 0) { count += 1; } } return count; }
int zip_items(int values[], int length) { int total = 0; for (int i = 0; i < length; i += 1) { total += values[i]; } return total; }
int flatten(int values[], int length) { int total = 0; for (int i = 0; i < length; i += 1) { for (int j = 0; j < length; j += 1) { total += values[j]; } } return total; }
int contains(int values[], int length, int target) { for (int i = 0; i < length; i += 1) { if (values[i] == target) { return 1; } } return 0; }
