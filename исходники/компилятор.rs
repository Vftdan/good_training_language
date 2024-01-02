/// Промежуточное Представление

use super::Результат;
use std::collections::HashMap;
use синтаксис::*;
use диагностика::*;
use лексика::*;
use типизация::*;

/// Инструкция промежуточного представления
#[derive(Debug)]
pub enum Инструкция {
    Ноп,
    /// Протолкнуть целое значение на стек аргументов.
    ПротолкнутьЦелое(usize),
    /// Протолкнуть указатель на данные.
    ///
    /// Эта инстуркция нужна потому, что мы не знаем во время
    /// компиляции где начинаются данные. Мы это только знаем во время
    /// интерпретации, либо генерации машинного кода.
    ПротолкнутьУказатель(usize), // СДЕЛАТЬ: по возможности, использовать u64 вместо usize для значений пп
    Вытолкнуть,
    СохранитьКадр,
    ВосстановитьКадр,
    ПротолкнутьОтКадра(usize),
    ВызватьПроцедуру(usize),
    Записать64,
    Прочитать64,
    ЦелСложение,
    ЦелМеньше,
    ЛогОтрицание,
    ПечатьСтроки,
    Возврат,
    Прыжок(usize),
    УсловныйПрыжок(usize),
}

#[derive(Clone)]
pub struct СкомпПеременная {
    pub имя: Лексема,
    pub тип: Тип,
    pub адрес: usize,
}

pub struct СкомпПараметр {
    pub имя: Лексема,
    pub тип: Тип,
}

pub struct СкомпПроцедура {
    pub имя: Лексема,
    pub параметры: Vec<СкомпПараметр>,
    pub точка_входа: usize,
}

#[derive(Debug)]
pub struct СкомпКонстанта {
    pub синтаксис: Константа,
    pub значение: usize,
}

/// Промежуточное Представление
#[derive(Default)]
pub struct ПП {
    pub код: Vec<Инструкция>,
    pub иниц_данные: Vec<u8>,
    pub размер_неиниц_данных: usize,
    pub заплатки_неиниц_указателей: Vec<usize>,
}

impl ПП {
    pub fn вывалить(&self, точка_входа: usize) {
        println!("Инструкции ({количество} {инструкций}):",
                 количество = self.код.len(),
                 инструкций = ЧИСУЩ_ИНСТРУКЦИЙ.текст(self.код.len()));
        let ширина_столбца_индекса = self.код.len().to_string().len();
        for (индекс, инструкция) in self.код.iter().enumerate() {
            print!("{индекс:0>ширина_столбца_индекса$}: {инструкция:?}");
            if точка_входа == индекс {
                print!(" <- точка входа");
            }
            println!()
        }
        println!();
        println!("Инициализированные данные ({размер} {байт}):",
                 размер = self.иниц_данные.len(),
                 байт = ЧИСУЩ_БАЙТ.текст(self.иниц_данные.len()));
        const ШИРИНА_КОЛОНКИ_ИНИЦ_ДАННЫХ: usize = 16;
        for строка in 0..self.иниц_данные.len()/ШИРИНА_КОЛОНКИ_ИНИЦ_ДАННЫХ {
            let адрес = строка*ШИРИНА_КОЛОНКИ_ИНИЦ_ДАННЫХ;
            print!("{адрес:#08X}: ");
            let байты = &self.иниц_данные[адрес..адрес + ШИРИНА_КОЛОНКИ_ИНИЦ_ДАННЫХ];
            for байт in байты {
                print!("{байт:#04X} ");
            }
            println!()
        }
        let остаток = self.иниц_данные.len()%ШИРИНА_КОЛОНКИ_ИНИЦ_ДАННЫХ;
        if остаток > 0 {
            let адрес = self.иниц_данные.len()/ШИРИНА_КОЛОНКИ_ИНИЦ_ДАННЫХ*ШИРИНА_КОЛОНКИ_ИНИЦ_ДАННЫХ;
            print!("{адрес:#08X}: ");
            let байты = &self.иниц_данные[адрес..адрес + остаток];
            for байт in байты {
                print!("{байт:#04X} ");
            }
            println!()
        }
        println!();
        println!("Размер неинициализированных данных: {размер} {байт}",
                 размер = self.размер_неиниц_данных,
                 байт = ЧИСУЩ_БАЙТ.текст(self.размер_неиниц_данных));
    }
}

#[derive(Default)]
pub struct Имена {
    pub константы: HashMap<String, СкомпКонстанта>,
    pub процедуры: HashMap<String, СкомпПроцедура>,
    pub переменные: HashMap<String, СкомпПеременная>,
}

impl Имена {
    fn верифицировать_переопределение_имени(&self, имя: &Лексема) -> Результат<()> {
        if let Some(существующая_переменная) = self.переменные.get(&имя.текст) {
            диагностика!(&имя.лок, "ОШИБКА",
                         "уже существует переменная с именем «{имя}»",
                         имя = имя.текст);
            диагностика!(&существующая_переменная.имя.лок, "ИНФО",
                         "она определена здесь здесь. Выберите другое имя.");
            return Err(())
        }

        if let Some(существующая_процедура) = self.процедуры.get(&имя.текст) {
            диагностика!(&имя.лок, "ОШИБКА",
                         "уже существует процедура с именем «{имя}»",
                         имя = имя.текст);
            диагностика!(&существующая_процедура.имя.лок, "ИНФО",
                         "она определена здесь здесь. Выберите другое имя.");
            return Err(())
        }

        if let Some(существующая_константа) = self.константы.get(&имя.текст) {
            диагностика!(&имя.лок, "ОШИБКА",
                         "уже существует константа с именем «{имя}»",
                         имя = имя.текст);
            диагностика!(&существующая_константа.синтаксис.имя.лок, "ИНФО",
                         "она определена здесь здесь. Выберите другое имя.");
            return Err(())
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct Программа {
    pub пп: ПП,
    pub имена: Имена,
}

fn скомпилировать_выражение(пп: &mut ПП, имена: &Имена, выражение: &Выражение) -> Результат<Тип> {
    match выражение {
        Выражение::Число(_, число) => {
            пп.код.push(Инструкция::ПротолкнутьЦелое(*число));
            Ok(Тип::ПримТип(ПримТип::Цел8))
        },
        Выражение::Строка(строка) => {
            let указатель = пп.иниц_данные.len();
            let длинна = строка.текст.len();
            пп.иниц_данные.extend(строка.текст.as_bytes());
            пп.код.push(Инструкция::ПротолкнутьЦелое(длинна));
            пп.код.push(Инструкция::ПротолкнутьУказатель(указатель));
            Ok(Тип::ПримТип(ПримТип::Строка))
        }
        Выражение::Идент(лексема) => {
            if let Some(константа) = имена.константы.get(&лексема.текст) {
                пп.код.push(Инструкция::ПротолкнутьЦелое(константа.значение));
                return Ok(Тип::ПримТип(ПримТип::Цел8));
            }
            if let Some(переменная) = имена.переменные.get(&лексема.текст) {
                пп.заплатки_неиниц_указателей.push(пп.код.len());
                пп.код.push(Инструкция::ПротолкнутьУказатель(переменная.адрес));
                match переменная.тип {
                    Тип::ПримТип(ПримТип::Цел1) => {
                        сделать!(&лексема.лок, "чтение {}", переменная.тип.текст());
                        return Err(())
                    }
                    Тип::ПримТип(ПримТип::Цел8) => {
                        пп.код.push(Инструкция::Прочитать64);
                        return Ok(переменная.тип.clone());
                    }
                    Тип::ПримТип(ПримТип::Лог) => {
                        сделать!(&лексема.лок, "чтение логических переменных");
                        return Err(())
                    }
                    Тип::ПримТип(ПримТип::Строка) => {
                        сделать!(&лексема.лок, "чтение строковых переменных");
                        return Err(())
                    }
                    Тип::Массив{..} => {
                        сделать!(&лексема.лок, "чтение массивов");
                        return Err(())
                    }
                    Тип::Срез{..} => {
                        сделать!(&лексема.лок, "чтение Срезов");
                        return Err(())
                    }
                }
            }
            диагностика!(&лексема.лок, "ОШИБКА",
                         "не существует ни констант, ни переменных с имением «{имя}»",
                         имя = &лексема.текст);
            Err(())
        }
        Выражение::Биноп {ключ, вид, левое, правое} => {
            let левый_тип = скомпилировать_выражение(пп, имена, &левое)?;
            let правый_тип = скомпилировать_выражение(пп, имена, &правое)?;
            match вид {
                ВидБинопа::Меньше => {
                    проверить_типы(левое.лок(), &Тип::ПримТип(ПримТип::Цел8), &левый_тип)?;
                    проверить_типы(правое.лок(), &Тип::ПримТип(ПримТип::Цел8), &правый_тип)?;
                    пп.код.push(Инструкция::ЦелМеньше);
                    Ok(Тип::ПримТип(ПримТип::Лог))
                }
                ВидБинопа::Сложение => {
                    проверить_типы(левое.лок(), &Тип::ПримТип(ПримТип::Цел8), &левый_тип)?;
                    проверить_типы(правое.лок(), &Тип::ПримТип(ПримТип::Цел8), &правый_тип)?;
                    пп.код.push(Инструкция::ЦелСложение);
                    Ok(Тип::ПримТип(ПримТип::Цел8))
                }
                ВидБинопа::Вычитание => {
                    сделать!(&ключ.лок, "Компиляция бинопа «вычитание»");
                    Err(())
                }
                ВидБинопа::Деление => {
                    сделать!(&ключ.лок, "Компиляция бинопа «деление»");
                    Err(())
                }
                ВидБинопа::Остаток => {
                    сделать!(&ключ.лок, "Компиляция бинопа «остаток»");
                    Err(())
                }
                ВидБинопа::Равно => {
                    сделать!(&ключ.лок, "Компиляция бинопа «равно»");
                    Err(())
                }
                ВидБинопа::Больше => {
                    сделать!(&ключ.лок, "Компиляция бинопа «больше»");
                    Err(())
                }
            }
        }

        Выражение::Вызов {имя, ..} => {
            сделать!(&имя.лок, "компиляция выражение вызова функции");
            Err(())
        }
    }
}

fn скомпилировать_утвержление(пп: &mut ПП, имена: &Имена, утверждение: &Утверждение) -> Результат<()> {
    match утверждение {
        Утверждение::Присваивание{имя, значение, ..} => {
            if let Some(переменная) = имена.переменные.get(имя.текст.as_str()) {
                let тип = скомпилировать_выражение(пп, имена, &значение)?;
                проверить_типы(&значение.лок(), &переменная.тип, &тип)?;
                пп.заплатки_неиниц_указателей.push(пп.код.len());
                пп.код.push(Инструкция::ПротолкнутьУказатель(переменная.адрес));
                пп.код.push(Инструкция::Записать64);
                Ok(())
            } else {
                диагностика!(&имя.лок, "ОШИБКА", "Неизвестная переменная «{имя}»", имя = имя.текст);
                return Err(())
            }
        }
        Утверждение::ПрисваиваниеМассива{имя, ..} => {
            сделать!(&имя.лок, "компиляция присваивания массива");
            return Err(())
        }
        Утверждение::Вызов{имя, аргументы} => {
            match имя.текст.as_str() {
                // СДЕЛАТЬ: не позволять переопределять процедуру «печать» в пользовательском коде.
                "печать" => {
                    for арг in аргументы {
                        let тип = скомпилировать_выражение(пп, имена, &арг)?;
                        match тип {
                            Тип::ПримТип(ПримТип::Строка) => пп.код.push(Инструкция::ПечатьСтроки),
                            Тип::ПримТип(ПримТип::Цел8) => {
                                сделать!(арг.лок(), "печать цел(8)");
                                return Err(())
                            }
                            Тип::ПримТип(ПримТип::Цел1) => {
                                сделать!(арг.лок(), "печать цел(1)");
                                return Err(())
                            }
                            Тип::ПримТип(ПримТип::Лог) => {
                                сделать!(арг.лок(), "печать логического типа");
                                return Err(())
                            }
                            Тип::Массив{..} => {
                                сделать!(арг.лок(), "печать массивов");
                                return Err(())
                            }
                            Тип::Срез{..} => {
                                сделать!(арг.лок(), "печать срезов");
                                return Err(())
                            }
                        }
                    }
                    Ok(())
                },
                _ => {
                    if let Some(процедура) = имена.процедуры.get(&имя.текст) {
                        let количество_аргументов = аргументы.len();
                        let количество_параметров = процедура.параметры.len();
                        if количество_аргументов != количество_параметров {
                            диагностика!(&имя.лок, "ОШИБКА",
                                         "Неверное количество аргументов вызова процедуры. Процедура принимает {количество_параметров} {параметров}, но в данном вызове предоставлено лишь {количество_аргументов} {аргументов}.",
                                         параметров = ЧИСУЩ_ПАРАМЕТР.текст(количество_параметров),
                                         аргументов = ЧИСУЩ_АРГУМЕНТ.текст(количество_аргументов));
                            return Err(());
                        }

                        пп.код.push(Инструкция::СохранитьКадр);
                        for (параметр, аргумент) in процедура.параметры.iter().zip(аргументы.iter()) {
                            let тип = скомпилировать_выражение(пп, имена, аргумент)?;
                            проверить_типы(&аргумент.лок(), &параметр.тип, &тип)?;
                            if параметр.тип != Тип::ПримТип(ПримТип::Цел8) {
                                сделать!(&параметр.имя.лок, "Определение локальных переменных типа «{тип:?}»", тип = параметр.тип);
                                return Err(())
                            }
                        }
                        пп.код.push(Инструкция::ВызватьПроцедуру(процедура.точка_входа));
                        for параметр in &процедура.параметры {
                            if параметр.тип == Тип::ПримТип(ПримТип::Цел8) {
                                пп.код.push(Инструкция::Вытолкнуть);
                            } else {
                                сделать!(&параметр.имя.лок, "Сброс локальных переменных типа «{тип:?}»", тип = параметр.тип);
                                return Err(())
                            }
                        }
                        пп.код.push(Инструкция::ВосстановитьКадр);
                        Ok(())
                    } else {
                        диагностика!(&имя.лок, "ОШИБКА", "Неизвестная процедура «{имя}»", имя = имя.текст);
                        Err(())
                    }
                }
            }
        }
        Утверждение::Если{ключ: _, условие, тело} => {
            let тип = скомпилировать_выражение(пп, имена, &условие)?;
            проверить_типы(&условие.лок(), &Тип::ПримТип(ПримТип::Лог), &тип)?;
            пп.код.push(Инструкция::ЛогОтрицание);
            let точка_условного_прыжка = пп.код.len();
            пп.код.push(Инструкция::Ноп);
            for утверждение in тело.iter() {
                скомпилировать_утвержление(пп, имена, утверждение)?;
            }
            let точка_выхода = пп.код.len();
            пп.код[точка_условного_прыжка] = Инструкция::УсловныйПрыжок(точка_выхода);
            Err(())
        }
        Утверждение::Вернуть{ключ} => {
            сделать!(&ключ.лок, "Компиляция конструкции «вернуть»");
            Err(())
        }
        Утверждение::Пока{ключ: _, условие, тело} => {
            let точка_условия = пп.код.len();
            let тип = скомпилировать_выражение(пп, имена, &условие)?;
            проверить_типы(&условие.лок(), &Тип::ПримТип(ПримТип::Лог), &тип)?;
            пп.код.push(Инструкция::ЛогОтрицание);
            let точка_условного_прыжка = пп.код.len();
            пп.код.push(Инструкция::Ноп);
            for утверждение in тело.iter() {
                скомпилировать_утвержление(пп, имена, утверждение)?;
            }
            пп.код.push(Инструкция::Прыжок(точка_условия));
            let точка_выхода = пп.код.len();
            пп.код[точка_условного_прыжка] = Инструкция::УсловныйПрыжок(точка_выхода);
            Ok(())
        }
    }
}

fn скомпилировать_параметр(константы: &HashMap<String, СкомпКонстанта>, параметр: Параметр) -> Результат<СкомпПараметр> {
    Ok(СкомпПараметр {
        имя: параметр.имя,
        тип: скомпилировать_тип(параметр.тип, константы)?
    })
}

fn скомпилировать_процедуру(пп: &mut ПП, имена: &Имена, процедура: Процедура) -> Результат<СкомпПроцедура> {
    let mut параметры = Vec::new();
    for параметр in процедура.параметры {
        параметры.push(скомпилировать_параметр(&имена.константы, параметр)?);
    }
    let точка_входа = пп.код.len();
    for утверждение in &процедура.тело {
        скомпилировать_утвержление(пп, имена, утверждение)?;
    }
    пп.код.push(Инструкция::Возврат);
    Ok(СкомпПроцедура{
        имя: процедура.имя,
        параметры,
        точка_входа
    })
}

fn интерпретировать_выражение_константы(константы: &HashMap<String, СкомпКонстанта>, выражение: &Выражение) -> Результат<usize> {
    match выражение {
        &Выражение::Число(_, число) => Ok(число),
        Выражение::Строка(строка) => {
            сделать!(&строка.лок, "строковые константы");
            Err(())
        }
        Выражение::Идент(имя) => {
            if let Some(константа) = константы.get(имя.текст.as_str()) {
                Ok(константа.значение)
            } else {
                диагностика!(&имя.лок, "ОШИБКА", "Неизвестная константа «{имя}»", имя = имя.текст);
                Err(())
            }
        }
        Выражение::Биноп{ключ, вид, левое, правое, ..} => {
            let левое_значение = интерпретировать_выражение_константы(константы, левое)?;
            let правое_значение = интерпретировать_выражение_константы(константы, правое)?;
            match вид {
                ВидБинопа::Равно | ВидБинопа::Меньше | ВидБинопа::Больше => {
                    сделать!(&ключ.лок, "булевые константы");
                    Err(())
                },
                ВидБинопа::Сложение => {
                    Ok(левое_значение + правое_значение)
                }
                ВидБинопа::Вычитание => {
                    Ok(левое_значение - правое_значение)
                }
                ВидБинопа::Деление => {
                    Ok(левое_значение / правое_значение)
                }
                ВидБинопа::Остаток => {
                    Ok(левое_значение % правое_значение)
                }
            }
        }
        Выражение::Вызов{имя, ..} => {
            сделать!(&имя.лок, "вызов функции в константном контексте");
            Err(())
        }
    }
}

fn скомпилировать_прим_тип(тип: ВыражениеПримТипа, константы: &HashMap<String, СкомпКонстанта>) -> Результат<ПримТип> {
match тип {
            ВыражениеПримТипа::ЦелРазмерное {имя: _, размер} => {
                let лок = размер.лок();
                let размер = интерпретировать_выражение_константы(константы, &размер)?;
                match размер {
                    1 => Ok(ПримТип::Цел1),
                    8 => Ok(ПримТип::Цел8),
                    _ => {
                        сделать!(&лок, "На данный момент размер целого может быть только 1 или 8");
                        Err(())
                    }
                }
            }
            ВыражениеПримТипа::Цел {имя: _} => {
                Ok(ПримТип::Цел8)
            }
            ВыражениеПримТипа::Лог {имя} => {
                сделать!(&имя.лок, "компиляция логического типа");
                Err(())
            }
            ВыражениеПримТипа::Строка {имя} => {
                сделать!(&имя.лок, "компиляция целого размерного типа");
                Err(())
            }
        }
}

fn скомпилировать_тип(тип: ВыражениеТипа, константы: &HashMap<String, СкомпКонстанта>) -> Результат<Тип> {
    match тип {
        ВыражениеТипа::ПримТип(тип) => Ok(Тип::ПримТип(скомпилировать_прим_тип(тип, константы)?)),
        ВыражениеТипа::Массив {размер, прим_тип, ..} => {
            Ok(Тип::Массив {
                размер: интерпретировать_выражение_константы(константы, &размер)?,
                прим_тип: скомпилировать_прим_тип(прим_тип, константы)?,
            })
        }
        ВыражениеТипа::Срез {прим_тип, ..} => {
            Ok(Тип::Срез {прим_тип: скомпилировать_прим_тип(прим_тип, константы)?})
        }
    }
}

impl Программа {
    pub fn скомпилировать_лексемы(&mut self, лекс: &mut Лексер) -> Результат<()> {
        loop {
            let ключ = лекс.вытащить_лексему_вида(&[
                ВидЛексемы::КлючПер,
                ВидЛексемы::КлючПро,
                ВидЛексемы::КлючКонст,
                ВидЛексемы::Конец,
            ])?;
            match ключ.вид {
                ВидЛексемы::КлючПер => {
                    let синтаксис = Переменная::разобрать(лекс)?;
                    let имя = синтаксис.имя;
                    self.имена.верифицировать_переопределение_имени(&имя)?;
                    let тип = скомпилировать_тип(синтаксис.тип, &self.имена.константы)?;
                    let адрес = self.пп.размер_неиниц_данных;
                    self.пп.размер_неиниц_данных += тип.размер();
                    if let Some(_) = self.имена.переменные.insert(имя.текст.clone(), СкомпПеременная {имя, тип, адрес}) {
                        unreachable!("Проверка переопределения переменных должна происходить на этапе разбора")
                    }
                }
                ВидЛексемы::КлючПро => {
                    let процедура = Процедура::разобрать(лекс)?;
                    self.имена.верифицировать_переопределение_имени(&процедура.имя)?;
                    let скомп_процедура = скомпилировать_процедуру(&mut self.пп, &self.имена, процедура)?;
                    if let Some(_) = self.имена.процедуры.insert(скомп_процедура.имя.текст.clone(), скомп_процедура) {
                        unreachable!("Проверка переопределения процедур должна происходить на этапе разбора")
                    }
                }
                ВидЛексемы::КлючКонст => {
                    let константа = Константа::разобрать(лекс)?;
                    self.имена.верифицировать_переопределение_имени(&константа.имя)?;
                    let значение = интерпретировать_выражение_константы(&self.имена.константы, &константа.выражение)?;
                    if let Some(_) = self.имена.константы.insert(константа.имя.текст.clone(), СкомпКонстанта { синтаксис: константа, значение }) {
                        unreachable!("Проверка переопределения констант должна происходить на этапе разбора")
                    }
                }
                ВидЛексемы::Конец => break,
                _ => unreachable!(),
            }
        }

        for индекс in &self.пп.заплатки_неиниц_указателей {
            if let Some(Инструкция::ПротолкнутьУказатель(указатель)) = self.пп.код.get_mut(*индекс) {
                *указатель += self.пп.иниц_данные.len();
            } else {
                unreachable!("Ошибка в процессе сбора заплаток указателей на неинициализированные данные. Каждый индекс такой заплатки должен указывать на инструкцию ПротолкнутьУказатель");
            }
        }
        Ok(())
    }
}
