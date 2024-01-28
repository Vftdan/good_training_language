#!/bin/sh

set -xe

./собрать.sh

mkdir -p ./сборка/примеры/
./сборка/хуяк комп -цель фазм -вывод ./сборка/примеры/01-привет-фазм    ./примеры/01-привет.хуя
./сборка/хуяк комп -цель фазм -вывод ./сборка/примеры/02-цикл-фазм      ./примеры/02-цикл.хуя
./сборка/хуяк комп -цель фазм -вывод ./сборка/примеры/03-имя-фазм       ./примеры/03-имя.хуя
./сборка/хуяк комп -цель фазм -вывод ./сборка/примеры/04-физз-базз-фазм ./примеры/04-физз-базз.хуя
./сборка/хуяк комп -цель фазм -вывод ./сборка/примеры/05-фибоначчи-фазм ./примеры/05-фибоначчи.хуя
./сборка/хуяк комп -цель фазм -вывод ./сборка/примеры/06-рейлиб-фазм    ./примеры/06-рейлиб.хуя

./сборка/примеры/01-привет-фазм
./сборка/примеры/02-цикл-фазм
echo 'Алексей' | ./сборка/примеры/03-имя-фазм
./сборка/примеры/04-физз-базз-фазм
./сборка/примеры/05-фибоначчи-фазм
#./сборка/примеры/06-рейлиб-фазм

./сборка/хуяк комп -цель эльф -вывод ./сборка/примеры/01-привет-эльф    ./примеры/01-привет.хуя
./сборка/хуяк комп -цель эльф -вывод ./сборка/примеры/02-цикл-эльф      ./примеры/02-цикл.хуя
./сборка/хуяк комп -цель эльф -вывод ./сборка/примеры/03-имя-эльф       ./примеры/03-имя.хуя
./сборка/хуяк комп -цель эльф -вывод ./сборка/примеры/04-физз-базз-эльф ./примеры/04-физз-базз.хуя
./сборка/хуяк комп -цель эльф -вывод ./сборка/примеры/05-фибоначчи-эльф ./примеры/05-фибоначчи.хуя
# ./сборка/хуяк комп -цель эльф -вывод ./сборка/примеры/06-рейлиб-эльф  ./примеры/06-рейлиб.хуя

./сборка/примеры/01-привет-эльф
./сборка/примеры/02-цикл-эльф
echo 'Алексей' | ./сборка/примеры/03-имя-эльф
./сборка/примеры/04-физз-базз-эльф
./сборка/примеры/05-фибоначчи-эльф
#./сборка/примеры/06-рейлиб-эльф

mkdir -p ./сборка/тесты/
./сборка/хуяк комп -цель фазм -пуск -вывод ./сборка/тесты/тест-фазм ./тесты/тест.хуя
# ./сборка/хуяк комп -цель эльф -пуск -вывод ./сборка/тесты/тест-эльф ./тесты/тест.хуя
