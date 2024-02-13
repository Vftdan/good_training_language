use std::io;
use std::io::{Read, Write, BufRead};
use std::collections::HashMap;
use std::convert::TryInto;
use super::Результат;
use std::mem;
use компилятор::{ПП, ВидИнструкции, Инструкция, СкомпПеременная};
use типизация::{Тип};

// Разметка памяти
// |    второй стек    | инициализированные данные | неинициализированные данные |    куча?    |
// ^                   ^
// 0                   Начало стека и данных. Стек растет в сторону нуля.

pub const РАЗМЕР_СЛОВА: usize = mem::size_of::<u64>();

#[derive(Default)]
pub struct Машина<'ы> {
    индекс_инструкции: usize,  // аналог rip
    кадр: usize,               // аналог rbp (для первого стека)
    второй_стек: usize,        // аналог rsp
    кадр_второго_стека: usize, // аналог rbp (для второго стека)
    // В каком-то смысле, эти переменные выше являются регистрами
    // нашей виртуальной машины, не смотря на то, что машина-то
    // стековая.

    стек: Vec<usize>,
    начало_данных: usize,
    начало_второго_стека: usize,
    память: Vec<u8>,
    инструкции: &'ы [Инструкция],
}

macro_rules! ошибка_времени_исполнения {
    ($машина:expr, $($аргы:tt)*) => {{
        let индекс_инструкции = $машина.индекс_инструкции;
        if let Some(инструкция) = $машина.инструкции.get(индекс_инструкции) {
            let вид_инструкции = &инструкция.вид;
            let ::диагностика::Лок{путь_к_файлу, строка, столбец} = &инструкция.лок;
            eprint!("{путь_к_файлу}:{строка}:{столбец}: {вид_инструкции:?}: ", путь_к_файлу = путь_к_файлу.display());
        }
        eprint!("ОШИБКА ВРЕМЕНИ ИСПОЛНЕНИЯ: {индекс_инструкции}: ");
        eprintln!($($аргы)*);
    }};
}

impl<'ы> Машина<'ы> {
    pub fn новая(пп: &ПП, объём_второго_стека: usize) -> Машина {
        let начало_данных = объём_второго_стека;
        let второй_стек = объём_второго_стека;
        let кадр_второго_стека = объём_второго_стека;
        let начало_второго_стека = объём_второго_стека;
        let mut машина = Машина {
            индекс_инструкции: 0,
            кадр: 0,
            второй_стек,
            кадр_второго_стека,
            стек: Vec::new(),

            начало_данных,
            начало_второго_стека,

            память: vec![],
            инструкции: &пп.код,
        };

        // СДЕЛАТЬ: Ресайз вектора капец какой медленный. Возможно из-за
        // инициализации. Надо что-нибудь с этим сделать.
        машина.память.resize(машина.память.len() + объём_второго_стека, 0);
        машина.память.extend_from_slice(пп.иниц_данные.as_slice());
        машина.память.resize(машина.память.len() + пп.размер_неиниц_данных, 0);
        машина
    }

    fn протолкнуть_значение(&mut self, значение: usize) -> Результат<()> {
        self.стек.push(значение);
        Ok(())
    }

    fn вытолкнуть_значение(&mut self) -> Результат<usize> {
        if let Some(значение) = self.стек.pop()  {
            Ok(значение)
        } else {
            ошибка_времени_исполнения!(self, "Опустошение стека");
            Err(())
        }
    }

    fn вытолкнуть_значения(&mut self, количество: usize) -> Результат<()> {
        for _ in 0..количество {
            let _ = self.вытолкнуть_значение()?;
        }
        Ok(())
    }

    fn срез_памяти(&mut self, адрес: usize, размер: usize) -> Результат<&mut [u8]> {
        let макс = self.память.len();
        if let Some(срез) = self.память.get_mut(адрес..адрес+размер) {
            Ok(срез)
        } else {
            ошибка_времени_исполнения!(self, "Попытка получить доступ к некорректнному диапазону памяти [{начало}..{конец}). Разрешенный диапазон [0..{макс})", начало = адрес, конец = адрес+размер);
            Err(())
        }
    }

    fn количество_элементов_стека(&self) -> usize {
        self.стек.len()
    }

    fn проверить_арность_аргументов(&self, арность: usize) -> Результат<()> {
        let размер_стека = self.количество_элементов_стека();
        if размер_стека < арность {
            ошибка_времени_исполнения!(self, "Недостаточно аргументов для инструкции. Требуется как минимум {арность}, но всего в стеке аргументов находится {размер_стека}.");
            Err(())
        } else {
            Ok(())
        }
    }

    fn инструкция(&self) -> Результат<&Инструкция> {
        match self.инструкции.get(self.индекс_инструкции) {
            Some(инструкция) => Ok(инструкция),
            None => {
                ошибка_времени_исполнения!(self, "некорректный индекс инструкции");
                Err(())
            }
        }
    }

    fn выделить_на_втором_стеке(&mut self, размер: usize) -> Результат<()> {
        if self.второй_стек < размер {
            ошибка_времени_исполнения!(self, "переполнение второго стека");
            return Err(())
        }
        self.второй_стек -= размер;
        Ok(())
    }

    fn освободить_со_второго_стека(&mut self, размер: usize) -> Результат<()> {
        if self.начало_второго_стека - self.второй_стек > размер {
            ошибка_времени_исполнения!(self, "опустошение второго стека");
            return Err(())
        }
        self.второй_стек += размер;
        Ok(())
    }

    fn протолкнуть_на_второй_стек(&mut self, значение: usize) -> Результат<()> {
        self.выделить_на_втором_стеке(РАЗМЕР_СЛОВА)?;
        self.срез_памяти(self.второй_стек, РАЗМЕР_СЛОВА)?.copy_from_slice(&значение.to_le_bytes());
        Ok(())
    }

    fn вытолкнуть_из_второго_стека(&mut self) -> Результат<usize> {
        let значение = usize::from_le_bytes(self.срез_памяти(self.второй_стек, РАЗМЕР_СЛОВА)?.try_into().unwrap());
        self.освободить_со_второго_стека(РАЗМЕР_СЛОВА)?;
        Ok(значение)
    }

    pub fn интерпретировать(&mut self, переменные: &HashMap<String, СкомпПеременная>, точка_входа: usize, режим_отладки: bool) -> Результат<()> {
        self.индекс_инструкции = точка_входа;

        let mut глубина_вызовов = 0;
        let mut цель_перешагивания: Option<usize> = None;
        loop {
            let индекс_инструкции = self.индекс_инструкции;
            let инструкция = self.инструкция()?;

            if режим_отладки {
                if let Some(цель) = цель_перешагивания.clone() {
                    if глубина_вызовов <= цель {
                        цель_перешагивания = None;
                    }
                }

                if цель_перешагивания.is_none() {
                    диагностика!(&инструкция.лок, "ИНСТРУКЦИЯ", "{индекс_инструкции}: {вид_инструкции:?}", вид_инструкции = инструкция.вид);
                    eprintln!("стек = {стек:?}", стек = self.стек);
                    eprintln!("кадр = {кадр}", кадр = self.кадр);
                    eprintln!("переменные");
                    for (имя, переменная) in переменные.iter() {
                        let адрес = переменная.адрес + self.начало_данных;
                        eprintln!("  {имя}: {адрес:#X} = {:?}", &self.память[адрес..адрес+переменная.тип.размер()], адрес = переменная.адрес + self.начало_данных);
                    }
                    loop {
                        let mut команда = String::new();
                        eprint!("> ");
                        io::stdin().lock().read_line(&mut команда).unwrap();
                        let аргы: Vec<&str> = команда.trim().split(' ').filter(|арг| арг.len() > 0).collect();
                        match аргы.as_slice() {
                            ["выход", ..] => {
                                return Ok(());
                            }
                            ["инст", парам @ ..] => match парам {
                                [инст] => match инст.parse::<usize>() {
                                    Ok(индекс_инструкции) => if let Some(инструкция) = self.инструкции.get(индекс_инструкции) {
                                        диагностика!(&инструкция.лок, "ИНСТРУКЦИЯ", "{индекс_инструкции}: {вид_инструкции:?}", вид_инструкции = инструкция.вид);
                                    } else {
                                        eprintln!("ОШИБКА: нету инструкции под номером {индекс_инструкции}")
                                    },
                                    Err(_ошибка) => {
                                        eprintln!("ОШИБКА: индекс инструкции не является корректным целым числом");
                                    },
                                },
                                _ => {
                                    eprintln!("Пример: инст [индекс_инструкции]");
                                    eprintln!("ОШИБКА: требуется индекс инструкции");
                                }
                            }
                            ["перешаг", ..] => {
                                цель_перешагивания = Some(глубина_вызовов);
                                break
                            }
                            [команда, ..] => {
                                eprintln!("ОШИБКА: неизвестная команда «{команда}»");
                            }
                            [] => {
                                break
                            }
                        }
                    }
                }
            }

            match &инструкция.вид {
                ВидИнструкции::Ноп => {
                    self.индекс_инструкции += 1;
                }
                &ВидИнструкции::ПротолкнутьУказатель(указатель) => {
                    self.протолкнуть_значение(указатель + self.начало_данных)?;
                    self.индекс_инструкции += 1;
                }
                &ВидИнструкции::ПротолкнутьЦел(значение)  => {
                    self.протолкнуть_значение(значение)?;
                    self.индекс_инструкции += 1;
                }
                &ВидИнструкции::Обменять => {
                    self.проверить_арность_аргументов(2)?;
                    let первое = self.вытолкнуть_значение()?;
                    let второе = self.вытолкнуть_значение()?;
                    self.протолкнуть_значение(первое)?;
                    self.протолкнуть_значение(второе)?;
                    self.индекс_инструкции += 1;
                }
                &ВидИнструкции::Вытолкнуть(количество) => {
                    self.проверить_арность_аргументов(количество)?;
                    self.вытолкнуть_значения(количество)?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::СохранитьКадр => {
                    self.протолкнуть_значение(self.кадр)?;
                    self.кадр = self.стек.len();
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::ВосстановитьКадр => {
                    self.проверить_арность_аргументов(1)?;
                    self.кадр = self.вытолкнуть_значение()?;
                    self.индекс_инструкции += 1;
                }
                &ВидИнструкции::ПрочитатьКадр(смещение) => {
                    self.протолкнуть_значение(self.стек[(self.кадр as i32 + смещение) as usize])?;
                    self.индекс_инструкции += 1;
                }
                &ВидИнструкции::ЗаписатьКадр(смещение) => {
                    self.проверить_арность_аргументов(1)?;
                    let значение = self.вытолкнуть_значение()?;
                    self.стек[(self.кадр as i32 + смещение) as usize] = значение;
                    self.индекс_инструкции += 1;
                }
                &ВидИнструкции::ВыделитьНаВторомСтеке(размер) => {
                    self.выделить_на_втором_стеке(размер as usize)?;
                    self.индекс_инструкции += 1;
                }
                &ВидИнструкции::ОсвободитьСоВторогоСтека(размер) => {
                    self.освободить_со_второго_стека(размер as usize)?;
                    self.индекс_инструкции += 1;
                }
                &ВидИнструкции::ВершинаВторогоСтека(смещение) => {
                    self.протолкнуть_значение((self.второй_стек as i32 + смещение) as usize)?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::СохранитьКадрВторогоСтека => {
                    let старый_кадр = self.кадр_второго_стека;
                    self.кадр_второго_стека = self.второй_стек;
                    self.протолкнуть_на_второй_стек(старый_кадр)?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::ВосстановитьКадрВторогоСтека => {
                    self.кадр_второго_стека = self.вытолкнуть_из_второго_стека()?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::КадрВторогоСтека(смещение) => {
                    self.протолкнуть_значение((self.кадр_второго_стека as i32 + смещение) as usize)?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::АргументНаСтек => {
                    let значение = self.вытолкнуть_значение()?;
                    self.протолкнуть_на_второй_стек(значение)?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::АргументСоСтека => {
                    let значение = self.вытолкнуть_из_второго_стека()?;
                    self.протолкнуть_значение(значение)?;
                    self.индекс_инструкции += 1;
                }
                &ВидИнструкции::ВнутреннийВызов(адрекс) => {
                    глубина_вызовов += 1;
                    self.протолкнуть_значение(индекс_инструкции + 1)?;
                    self.индекс_инструкции = адрекс;
                }
                ВидИнструкции::ВнешнийВызов{..} => {
                    ошибка_времени_исполнения!(self, "вынешние вызовы не поддерживаются в режиме интерпретации");
                    return Err(())
                }
                ВидИнструкции::Записать8 => {
                    self.проверить_арность_аргументов(2)?;
                    let адрес = self.вытолкнуть_значение()?;
                    let значение = (self.вытолкнуть_значение()? & 0xFF) as u8;
                    let тип = Тип::Цел8;
                    self.срез_памяти(адрес, тип.размер())?.copy_from_slice(&значение.to_le_bytes());
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::Записать32 => {
                    сделать!(&инструкция.лок, "Интерпретация инструкции Записать32");
                    return Err(());
                }
                ВидИнструкции::Записать64 => {
                    self.проверить_арность_аргументов(2)?;
                    let адрес = self.вытолкнуть_значение()?;
                    let значение = self.вытолкнуть_значение()?;
                    let тип = Тип::Цел64;
                    self.срез_памяти(адрес, тип.размер())?.copy_from_slice(&значение.to_le_bytes());
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::Прочитать32 => {
                    сделать!(&инструкция.лок, "Интерпртация инструкции Прочитать32");
                    return Err(());
                }
                ВидИнструкции::Прочитать64 => {
                    self.проверить_арность_аргументов(1)?;
                    let адрес = self.вытолкнуть_значение()?;
                    let тип = Тип::Цел64;
                    let значение: u64 = u64::from_le_bytes(self.срез_памяти(адрес, тип.размер())?.try_into().unwrap());
                    self.протолкнуть_значение(значение as usize)?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::СкопироватьПамять => {
                    сделать!(&инструкция.лок, "Реализовать интерпретацию инстуркции {вид_инструкции:?}", вид_инструкции = инструкция.вид);
                    return Err(())
                }
                ВидИнструкции::ЦелМеньше => {
                    self.проверить_арность_аргументов(2)?;
                    let правый = self.вытолкнуть_значение()?;
                    let левый = self.вытолкнуть_значение()?;
                    if левый < правый {
                        self.протолкнуть_значение(1)?;
                    } else {
                        self.протолкнуть_значение(0)?;
                    }
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::ЦелБольше => {
                    self.проверить_арность_аргументов(2)?;
                    let правый = self.вытолкнуть_значение()?;
                    let левый = self.вытолкнуть_значение()?;
                    if левый > правый {
                        self.протолкнуть_значение(1)?;
                    } else {
                        self.протолкнуть_значение(0)?;
                    }
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::ЦелРавно => {
                    self.проверить_арность_аргументов(2)?;
                    let правый = self.вытолкнуть_значение()?;
                    let левый = self.вытолкнуть_значение()?;
                    if левый == правый {
                        self.протолкнуть_значение(1)?;
                    } else {
                        self.протолкнуть_значение(0)?;
                    }
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::ЦелСложение => {
                    self.проверить_арность_аргументов(2)?;
                    let правый = self.вытолкнуть_значение()?;
                    let левый = self.вытолкнуть_значение()?;
                    self.протолкнуть_значение(левый + правый)?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::ЦелВычитание => {
                    self.проверить_арность_аргументов(2)?;
                    let правый = self.вытолкнуть_значение()?;
                    let левый = self.вытолкнуть_значение()?;
                    self.протолкнуть_значение(левый - правый)?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::ЦелУмножение => {
                    self.проверить_арность_аргументов(2)?;
                    let правый = self.вытолкнуть_значение()?;
                    let левый = self.вытолкнуть_значение()?;
                    self.протолкнуть_значение(левый * правый)?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::ЦелДеление => {
                    self.проверить_арность_аргументов(2)?;
                    let правый = self.вытолкнуть_значение()?;
                    let левый = self.вытолкнуть_значение()?;
                    self.протолкнуть_значение(левый / правый)?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::ЦелОстаток => {
                    self.проверить_арность_аргументов(2)?;
                    let правый = self.вытолкнуть_значение()?;
                    let левый = self.вытолкнуть_значение()?;
                    self.протолкнуть_значение(левый % правый)?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::КонвертЦел64Вещ32 => {
                    сделать!(&инструкция.лок, "Интерпретация инструкции КонвертЦел64Вещ32");
                    return Err(());
                }
                ВидИнструкции::КонвертВещ32Цел64 => {
                    сделать!(&инструкция.лок, "Интерпретация инструкции КонвертВещ32Цел64");
                    return Err(());
                }
                ВидИнструкции::Вещ32Умножение => {
                    сделать!(&инструкция.лок, "Интерпретация инструкции Вещ32Умножение");
                    return Err(());
                }
                ВидИнструкции::Вещ32Деление => {
                    сделать!(&инструкция.лок, "Интерпретация инструкции Вещ32Деление");
                    return Err(());
                }
                ВидИнструкции::Вещ32Сложение => {
                    сделать!(&инструкция.лок, "Интерпретация инструкции Вещ32Сложение");
                    return Err(());
                }
                ВидИнструкции::Вещ32Меньше => {
                    сделать!(&инструкция.лок, "Интерпретация инструкции Вещ32Меньше");
                    return Err(());
                }
                ВидИнструкции::Вещ32Больше => {
                    сделать!(&инструкция.лок, "Интерпретация инструкции Вещ32Больше");
                    return Err(());
                }
                ВидИнструкции::Вещ32Инверт => {
                    сделать!(&инструкция.лок, "Интерпретация инструкции Вещ32Инверт");
                    return Err(());
                }
                ВидИнструкции::ЛогОтрицание => {
                    self.проверить_арность_аргументов(1)?;
                    let значение = self.вытолкнуть_значение()?;
                    if значение == 0 {
                        self.протолкнуть_значение(1)?;
                    } else {
                        self.протолкнуть_значение(0)?;
                    }
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::БитИли => {
                    сделать!(&инструкция.лок, "Интерпретация инструкции БитИли");
                    return Err(());
                }
                ВидИнструкции::Прыжок(индекс) => {
                    self.индекс_инструкции = *индекс;
                }
                &ВидИнструкции::УсловныйПрыжок(индекс) => {
                    self.проверить_арность_аргументов(1)?;
                    let значение = self.вытолкнуть_значение()?;
                    if значение == 0 {
                        self.индекс_инструкции += 1;
                    } else {
                        self.индекс_инструкции = индекс;
                    }
                }
                ВидИнструкции::ПечатьСтроки => {
                    self.проверить_арность_аргументов(2)?;
                    let указатель = self.вытолкнуть_значение()?;
                    let длинна = self.вытолкнуть_значение()?;
                    let _ = io::stdout().write(self.срез_памяти(указатель, длинна)?);
                    let _ = io::stdout().flush();
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::Ввод => {
                    self.проверить_арность_аргументов(2)?;
                    let длинна = self.вытолкнуть_значение()?;
                    let указатель = self.вытолкнуть_значение()?;
                    let размер = io::stdin().read(self.срез_памяти(указатель, длинна)?).unwrap();
                    self.протолкнуть_значение(размер)?;
                    self.индекс_инструкции += 1;
                }
                ВидИнструкции::Возврат => {
                    // СДЕЛАТЬ: Ввести отдельную инструкцию останова.
                    // И генерировать точку входа наподобии того, как мы это делаем в эльф.
                    // Т.е. точка входа 0. Он прыгает в главную, и после вызывает останов.
                    if self.количество_элементов_стека() == 0 {
                        break;
                    }
                    self.индекс_инструкции = self.вытолкнуть_значение()?;
                    глубина_вызовов -= 1;
                },
                ВидИнструкции::СисВызов {..} => {
                    ошибка_времени_исполнения!(self, "системные вызовы не поддерживаются в режиме интерпретации");
                    return Err(())
                }
                ВидИнструкции::Стоп => {
                    break;
                }
            }
        }
        Ok(())
    }
}
