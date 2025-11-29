#include <ctype.h>
#include <fcntl.h>
#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#define MAX_FILE_SIZE 128
#define MAX_FILE_COUNT 1024
#define ARGUMENT_COUNT 5
#define DEFAULT_ARRAY_SIZE 512
#define SYMBOLS_TYPE 4
#define PASSWORD_NUM 2000
#define FILE_CONTENT_SIZE 7000000
#define MAX_PASSWORD_LENGTH 100
#define NUM_SUBSTITUTIONS 11
#define MAX_VALUE 0x3f3f3f3f
#define POWER_BASE 10
#define VERY_WEAK_THRESHOLD 35
#define WEAK_THRESHOLD 60
#define STRONG_THRESHOLD 120
#define SINGLE_BASE 2
#define DOUBLE_BASE 3

typedef enum {
  NO_STRONG_PASSWORD_ERROR = 8,
  FILE_READ_ERROR = 10,
  CMD_INVALID_ERROR = 11,
} ExitStatus;
typedef struct ListNode {
  struct ListNode* next;
  char* value;
} ListNode;

typedef struct List {
  ListNode* head;
  ListNode* tail;
  unsigned int len;
} List;

/*
 * add_node()
 * --------------
 * add node to list
 * */
void add_node(char* value, List* list) {
  ListNode* node = (ListNode*)malloc(sizeof(ListNode));
  node->next = NULL;
  size_t len = strlen(value);
  node->value = (char*)malloc((len + 1) * sizeof(char));
  strcpy(node->value, value);
  if (list->tail == NULL) {
    list->tail = list->head = node;
  } else {
    list->tail->next = node;
    list->tail = node;
  }
  //            tail    
  // [hello]-> [world]  
  list->len++;
}
void free_list(List* list) {
  ListNode* node = list->head;
  while (node != NULL) {
    ListNode* next = node->next;
    free(node->value);
    free(node);
    node = next;
  }
  list->head = list->tail = NULL;
  list->len = 0;
}
/*
 * print_usage()
 * --------------
 * print the usage
 * */
void print_usage(void) {
  fprintf(stderr,
          "Usage: ./uqentropy [--digit-append 1..7] [--double] [--leet] [--case] [filename ...]\n");
  exit(CMD_INVALID_ERROR);
}

/*
 * is_integer()
 * --------------
 * check a string is integer
 * */
bool is_integer(char* str) {
  int len = strlen(str);
  if (len == 0) {
    return false;
  }
  for (int i = 0; i < len; i++) {
    if (str[i] < '0' || str[i] > '9') {
      return false;
    }
  }
  return true;
}
/*
 * is_contain_non_printable_characters()
 * --------------
 * check a string is contain non-printable characters
 */
bool is_contain_non_printable_characters(char* str) {
  int len = strlen(str);
  for (int i = 0; i < len; i++) {
    if (isspace(str[i])) {
      continue;
    } else if (!isprint(str[i])) {
      return true;
    }
  }
  return false;
}
/*
 * is_valid_password()
 * --------------
 * check a string is valid password
 * */
bool is_contain_valid_password(char* str) {
  int len = strlen(str);
  for (int i = 0; i < len; i++) {
    if (!isspace(str[i])) {
      return true;
    }
  }
  return false;
}
/*
 * entropy_calculation_1()
 * --------------
 * calculation entropy by the first way
 */
double entropy_calculation_1(char* str) {
  int len = strlen(str);
  bool isUsed[SYMBOLS_TYPE];
  int weight[SYMBOLS_TYPE] = {10, 26, 26, 32};
  memset(isUsed, 0, sizeof(isUsed));
  for (int i = 0; i < len; i++) {
    if (isdigit(str[i])) {
      isUsed[0] = 1;
    } else if (str[i] >= 'a' && str[i] <= 'z') {
      isUsed[1] = 1;
    } else if (str[i] >= 'A' && str[i] <= 'Z') {
      isUsed[2] = 1;
    } else {
      isUsed[3] = 1;
    }
  }
  int sValue = 0;
  for (int i = 0; i < SYMBOLS_TYPE; i++) {
    if (isUsed[i]) {
      sValue += weight[i];
    }
  }
  int lValue = len;
  return log2(pow(sValue, lValue));
}

/*
 * get_letterCount()
 * --------------
 * get the number of letters in a string
 */
int get_letterCount(char* s) {
  int len = strlen(s);
  int count = 0;
  for (int i = 0; i < len; i++) {
    if (isalpha(s[i])) {
      count++;
    }
  }
  return count;
}
/*
 * print_found_message()
 * --------------
 * print the found message
 */
void print_found_message(int n) {
  fprintf(stdout, "Password matched on guess number %d\n", n);
}
int append_numbers(char* baseStr, int n, char* candidate) {
  int baseLen = strlen(baseStr);
  int maxNum = 1;

  for (int i = 0; i < n; i++) {
    maxNum *= POWER_BASE;
  }

  char* newStr = malloc(baseLen + n + 1);
  if (newStr == NULL) {
    printf("allocate failed\n");
    return 0;
  }

  for (int i = 0; i < maxNum; i++) {
    sprintf(newStr, "%s%0*d", baseStr, n, i);
    if (strcmp(candidate, newStr) == 0) {
      free(newStr);
      return i + 1;
    }
  }

  free(newStr);
  return 0;
}

typedef struct {
  char original;
  char substitutes[3];
} LeetSubstitution;

/*
 *generate_leet_combinations
 * --------------
 * generate leet combinations
 */
bool generate_leet_combinations(const char* password, int* matchNum, char* to_check) {
  LeetSubstitution leetTable[NUM_SUBSTITUTIONS] = {
      {'a', "@4"}, {'b', "68"}, {'e', "3"},  {'g', "69"}, {'i', "1!"}, {'l', "1"},
      {'o', "0"},  {'s', "5$"}, {'t', "7+"}, {'x', "%"},  {'z', "2"}};
  int length = strlen(password); // 遍历所有字母变形后的情况  
  int combinations = 1;
  int substituteCount[MAX_PASSWORD_LENGTH] = {0};
  int singleCount = 0, doubleCount = 0;
  for (int i = 0; i < length; i++) {
    substituteCount[i] = 1;
    // aos 
    // singleCount = 1
    // doubleCount = 2
    for (int j = 0; j < NUM_SUBSTITUTIONS; j++) {
      if (tolower(password[i]) == leetTable[j].original) {
        int t = strlen(leetTable[j].substitutes);
        if (t == 1) {
          singleCount++;
        } else {
          doubleCount++;
        }
        substituteCount[i] += t;
        combinations *= substituteCount[i];
        break;
      }
    }
  }
  if (singleCount == 0 && doubleCount == 0) {
    return false;
  }
  *matchNum += pow(SINGLE_BASE, singleCount) * pow(DOUBLE_BASE, doubleCount) - 1;
  for (int combo = 0; combo < combinations; combo++) {
    char candidate[MAX_PASSWORD_LENGTH];
    strcpy(candidate, password);
    int currentCombo = combo;

    for (int i = 0; i < length; i++) {
      if (substituteCount[i] > 1) {
        int index = currentCombo % substituteCount[i];
        currentCombo /= substituteCount[i];

        if (index > 0) {
          for (int j = 0; j < NUM_SUBSTITUTIONS; j++) {
            if (tolower(password[i]) == leetTable[j].original) {
              candidate[i] = leetTable[j].substitutes[index - 1];
              break;
            }
          }
        }
      }
    }

    if (strcmp(candidate, to_check) == 0) {
      return true;
    }
  }
  return false;
}
/*
 * do_double_check
 * --------------
 * check the double combination
 */
double do_double_check(List* gList, char* str, int* checkedPasswords, int* found) {
  int candidateLen = strlen(str);
  ListNode* cursor = gList->head;
  for (int i = 0; cursor; cursor = cursor->next, i++) {
    ListNode* innerCursor = gList->head;
    int len1 = strlen(cursor->value);
    if (len1 > candidateLen || cursor->value[0] != str[0]) {
      *checkedPasswords += gList->len;
      continue;
    }
    for (int j = 0; innerCursor; innerCursor = innerCursor->next, j++) {
      *checkedPasswords+=1;
      int len2 = strlen(innerCursor->value);
      if (len1 + len2 != candidateLen) {
        continue;
      }

      char* newStr = malloc(len1 + len2 + 1);
      sprintf(newStr, "%s%s", cursor->value, innerCursor->value);
      if (strcmp(newStr, str) == 0) {
        print_found_message(*checkedPasswords);
        free(newStr);
        *found = 1;
        return log2(2 * (*checkedPasswords));
      }
      free(newStr);
    }
  }
  return 0;
}
double do_digit_append_check(char* str, List* gList, int* checkedPasswords, int digitAppend,
                             int* found) {
  int powerTable[7] = {10, 100, 1000, 10000, 100000, 1000000, 10000000};
  for (ListNode* cursor = gList->head; cursor; cursor = cursor->next) {
    int len = strlen(cursor->value);
    char lastChar = cursor->value[len - 1];
    if (!isdigit(lastChar)) {
      for (int j = 0; j < digitAppend; j++) {
        int ret = append_numbers(cursor->value, j + 1, str);
        if (ret != 0) {
          *checkedPasswords += ret;
          *found = 1;
          print_found_message(*checkedPasswords);
          return log2(SINGLE_BASE * (*checkedPasswords));
        }
        *checkedPasswords += powerTable[j];
      }
    }
  }
  return 0;
}
double do_case_check(List* gList, char* str, int* checkedPasswords, int* found) {
  for (ListNode* cursor = gList->head; cursor; cursor = cursor->next) {
    int letterCount = get_letterCount(cursor->value);
    *checkedPasswords += (int)(pow(2, letterCount)) - 1;
    if (strcasecmp(str, cursor->value) == 0) {
      print_found_message(*checkedPasswords);
      *found = 1;
      return log2(2 * (*checkedPasswords));
    }
  }
  return 0;
}
double do_leet_check(List* gList, char* str, int* checkedPasswords, int* found) {
  for (ListNode* cursor = gList->head; cursor; cursor = cursor->next) {
    int matchNum = 0;
    int ret = generate_leet_combinations(cursor->value, &matchNum, str);
    *checkedPasswords += matchNum;
    if (ret) {
      print_found_message(*checkedPasswords);
      *found = 1;
      return log2(2 * (*checkedPasswords));
    }
  }
  return 0;
}
double do_basic_check(List* gList, char* str, int* found, int* checkedPasswords) {
  ListNode* cursor = gList->head;
  for (int i = 0; cursor; i++) {
    *checkedPasswords += 1;
    if (strcmp(str, cursor->value) == 0) {
      print_found_message(*checkedPasswords);
      *found = 1;
      return log2(SINGLE_BASE * (i + 1));
    }
    cursor = cursor->next;
  }
  return 0;
}
/*
 *entropy_calculation_2
 * --------------
 * calculate the entropy in the second way
 */
double entropy_calculation_2(char* str, int isPresent[], int digitAppend, List* gList) {
  // 1. check exact match
  int found = 0;
  int checkedPasswords = 0;
  double result = do_basic_check(gList, str, &found, &checkedPasswords);
//   printf("do basic = %d\n", checkedPasswords);
  if (found) {
    return result;
  }
  // 2. ignore case check， --case
  if (isPresent[3]) {
    found = 0;
    result = do_case_check(gList, str, &checkedPasswords, &found);
    if (found) {
      return result;
    }
  }
  // 3. --digit-append,
  if (isPresent[0]) {
    found = 0;
    result = do_digit_append_check(str, gList, &checkedPasswords, digitAppend, &found);
    if (found) {
      return result;
    }
  }
  // 4. --double
  if (isPresent[1]) {
    found = 0;
    result = do_double_check(gList, str, &checkedPasswords, &found);
    if (found) {
      return result;
    }
  }
  // 5. --leet
  if (isPresent[2]) {
    found = 0;
    result = do_leet_check(gList, str, &checkedPasswords, &found);
    if (found) {
      return result;
    }
  }
  fprintf(stdout, "Would not find a match after checking %d passwords\n", checkedPasswords);
  return MAX_VALUE;
}
/*
 *is_valid_password()
 * --------------
 * check whether the password is valid
 *
 */
bool is_valid_password(char* str) {
  // 1. at least one character
  int len = strlen(str);
  if (len == 0) {
    return false;
  }
  // 2. only contain printable characters
  if (is_contain_non_printable_characters(str)) {
    return false;
  }
  // 3. not contain any whitespace
  for (int i = 0; i < len; i++) {
    if (isspace(str[i])) {
      return false;
    }
  }
  return true;
}

void do_entropy_calculation(int isPresent[], char* enteredPassword, int digitAppend, List* gList,
                            int* foundStrong) {
  double entropyOne = entropy_calculation_1(enteredPassword);
  double entropyTwo = MAX_VALUE;
  if (isPresent[4] == 1) {
    entropyTwo = entropy_calculation_2(enteredPassword, isPresent, digitAppend, gList);
  }
  double totalEntropy = entropyOne > entropyTwo ? entropyTwo : entropyOne;
  totalEntropy = floor(totalEntropy * POWER_BASE) / POWER_BASE; // 向下取整 保留一位小数

  fprintf(stdout, "Password entropy calculated to be %.1f\n", totalEntropy);

  // check the strength of the password
  if (totalEntropy < VERY_WEAK_THRESHOLD) {
    fprintf(stdout, "Password strength rating: very weak\n");
  } else if (totalEntropy < WEAK_THRESHOLD) {
    fprintf(stdout, "Password strength rating: weak\n");
  } else if (totalEntropy < STRONG_THRESHOLD) {
    fprintf(stdout, "Password strength rating: strong\n");
    *foundStrong += 1;
  } else {
    fprintf(stdout, "Password strength rating: very strong\n");
    *foundStrong += 1;
  }
}

void check_the_arguments(int argc, char* argv[], int isPresent[], int* digitAppend,
                         char filename[MAX_FILE_COUNT][MAX_FILE_SIZE], int* actualFilenameNum) {
  for (int j = 1; j < argc; j += 1) {
    bool flag = false;
    if (strcmp(argv[j], "--digit-append") == 0) {
      if (flag == true || j + 1 >= argc) {
        print_usage();
      }
      if (!is_integer(argv[j + 1])) {
        print_usage();
      } 
      *digitAppend = atoi(argv[j + 1]);
      if (*digitAppend < 1 || *digitAppend > 7) {
        print_usage();
      }
      flag = true;
      j += 1;
      continue;
    } else if (strcmp(argv[j], "--double") == 0) {
      if (isPresent[1] == 1) {
        print_usage();
      }
      isPresent[1] = 1;
      continue;
    } else if (strcmp(argv[j], "--leet") == 0) {
      if (isPresent[2] == 1) {
        print_usage();
      }
      isPresent[2] = 1;
      continue;
    } else if (strcmp(argv[j], "--case") == 0) {
      if (isPresent[3] == 1) {
        print_usage();
      }
      isPresent[3] = 1;
      continue;
    } else {
      int len = strlen(argv[j]);
      if (len >= 2 && argv[j][0] == '-' && argv[j][1] == '-') {
        print_usage();
      }
      isPresent[4] = 1;
      for (int k = j; k < argc; k++) {
        if (strlen(argv[k]) == 0) {
          print_usage();
        }
        strcpy(filename[*actualFilenameNum], argv[k]);
        *actualFilenameNum += 1;
      }
      break;
    }
  }
}
void split_and_process(char* content, void* gList) {
  char tempStr[BUFSIZ];
  int len = strlen(content);
  memset(tempStr, 0, sizeof(tempStr));
  for (int i = 0; i < len; i++) {
    if (isspace(content[i])) 
      continue;
    int j = 0;
    while (i + j < len && !isspace(content[i + j])) {
      tempStr[j] = content[i + j];
      j++;
    }
    i += j;
    add_node(tempStr, gList);
    memset(tempStr, 0, sizeof(tempStr));
  }
}
int main(int argc, char* argv[]) {
  List* gList = NULL;
  int actualFilenameNum = 0;
  int digitAppend = 0;
  char filename[MAX_FILE_COUNT][MAX_FILE_SIZE];

  memset(filename, 0, sizeof(filename));
  int isPresent[ARGUMENT_COUNT];
  memset(isPresent, 0, sizeof(isPresent));

  check_the_arguments(argc, argv, isPresent, &digitAppend, filename, &actualFilenameNum);

  // parsed all the arguments
  if ((isPresent[0] == 1 || isPresent[1] == 1 || isPresent[2] == 1 || isPresent[3] == 1) &&
      isPresent[4] == 0) {
    print_usage();
  }

  gList = (List*)malloc(sizeof(List));
  gList->head = NULL;
  gList->tail = NULL;

  // 2. file check
  bool isOk = true;
  for (int i = 0; i < actualFilenameNum; i++) {
    FILE* fp = fopen(filename[i], "r");
    if (fp == NULL) {
      isOk = false;
      fprintf(stderr, "uqentropy: unable to read from password file \"%s\"\n", filename[i]);
      continue;
    }
    // 2.1 read content of the file
    char* content;
    content = (char*)malloc(FILE_CONTENT_SIZE);
    fread(content, 1, FILE_CONTENT_SIZE - 1, fp);
    // 2.2 check the content
    if (is_contain_non_printable_characters(content)) {
      fprintf(stderr, "uqentropy: non-printable character found in file \"%s\"\n", filename[i]);
      isOk = false;
      free(content);
      continue;
    }

    if (!is_contain_valid_password(content)) {
      fprintf(stderr, "uqentropy: no passwords in file \"%s\"\n", filename[i]);
      isOk = false;
      free(content);
      continue;
    }

    split_and_process(content, gList);
    free(content);
    fclose(fp);
  }
  if (!isOk || (actualFilenameNum != 0 && gList->len == 0)) {

    exit(FILE_READ_ERROR);
  }
  int foundStrong = 0;
  fprintf(stdout, "Welcome to UQentropy!\nWritten by s4767301.\nEnter possible password to check "
                  "its strength.\n");
  char enteredPassword[DEFAULT_ARRAY_SIZE];
  while (true) {
    char* ret = fgets(enteredPassword, sizeof(enteredPassword), stdin);
    if (ret != NULL) {
      enteredPassword[strcspn(enteredPassword, "\n")] = 0;
    }
    if (ret == NULL) {
      if (feof(stdin)) {
        free(gList);
        
        if (foundStrong > 0) {
          exit(EXIT_SUCCESS);
        } else {
          fprintf(stdout, "No strong password(s) have been entered\n");
          exit(NO_STRONG_PASSWORD_ERROR);
        }
      } else {
        puts("finished");
      }
    }

    if (!is_valid_password(enteredPassword)) {
      fprintf(stderr, "Invalid password\n");
      continue;
    }
    do_entropy_calculation(isPresent, enteredPassword, digitAppend, gList, &foundStrong);
  }
  return 0;
}
