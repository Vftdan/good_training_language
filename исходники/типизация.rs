use super::Результат;
use std::collections::HashMap;
use диагностика::*;
use лексика::*;

// СДЕЛАТЬ: Реформировать термины используемые в типах
// - цел - знаковое целое число, Си аналог: int
// - нат - натуральное, беззнаковое цело число, Си аналог: unsigned int
// - вещ - вещественное, число с плавающей точкой формата IEEE 754, Си аналог: float, double, etc
// - лог - логический булевый тип, Си аналог: bool

pub struct Поле {
    pub имя: Лексема,
    pub тип: Тип,
    pub смещение: usize,
}

pub struct Структура {
    pub имя: Лексема,
    pub размер: usize,
    pub поля: HashMap<String, Поле>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Тип {
    Нат8,
    Нат64,
    Цел64,
    Вещ32,
    Строка,
    Лог,
    Массив { размер: usize, тип_элемента: Box<Тип> },
    Срез { тип_элемента: Box<Тип> },
    Структура(String),
}

pub const СРЕЗ_РАЗМЕР_СМЕЩЕНИЕ: usize = 0;
pub const СРЕЗ_АДРЕС_СМЕЩЕНИЕ: usize = 8;

impl Тип {
    pub fn примитивный(&self) -> bool {
        match self {
            Тип::Цел64 | Тип::Нат8 | Тип::Нат64 | Тип::Вещ32 | Тип::Лог => true,
            Тип::Строка | Тип::Массив {..} | Тип::Срез {..} | Тип::Структура {..} => false,
        }
    }

    pub fn примитивное_знаковое_чтение(&self) -> Option<bool> {
        match self {
            Тип::Цел64 => Some(true),
            Тип::Нат8 | Тип::Нат64 | Тип::Вещ32 | Тип::Лог => Some(false),
            Тип::Строка | Тип::Массив {..} | Тип::Срез {..} | Тип::Структура {..} => None,
        }
    }

    pub fn текст(&self) -> String {
        match self {
            Тип::Цел64 => "цел64".to_string(),
            Тип::Нат8 => "нат8".to_string(),
            Тип::Нат64 => "нат64".to_string(),
            Тип::Вещ32 => "вещ32".to_string(),
            Тип::Строка => "строка".to_string(),
            Тип::Лог => "лог".to_string(),
            Тип::Массив {тип_элемента, размер} => format!("массив({размер}, {тип_элемента})", тип_элемента = тип_элемента.текст()),
            Тип::Срез {тип_элемента} => format!("срез({тип_элемента})", тип_элемента = тип_элемента.текст()),
            Тип::Структура(имя) => имя.clone(),
        }
    }

    pub fn размер(&self, структуры: &HashMap<String, Структура>) -> usize {
        match self {
            Тип::Нат8 => 1,
            Тип::Нат64 => 8,
            Тип::Цел64 => 8,
            Тип::Вещ32 => 4,
            Тип::Строка => 16, // Два 64-х битных числа: указатель на начало и размер.
            Тип::Лог => 8,
            Тип::Массив {тип_элемента, размер} => тип_элемента.размер(структуры) * размер,
            Тип::Срез {..} => 16, // Два 64-х битных числа: указатель на начало и размер.
            Тип::Структура (имя) => {
                структуры
                    .get(имя)
                    .expect("Существование структуры должно быть уже проверено на этапе компиляции типа")
                    .размер
            }
        }
    }
}

pub fn проверить_типы(лок: &Лок, ожидаемый_тип: &Тип, действительный_тип: &Тип) -> Результат<()> {
    if ожидаемый_тип == действительный_тип {
        Ok(())
    } else {
        диагностика!(лок, "ОШИБКА", "Несоответствие типов данных. Ожидался тип «{ожидаемый}», но повстречался тип «{действительный}»",
                     ожидаемый = ожидаемый_тип.текст(),
                     действительный = действительный_тип.текст());
        Err(())
    }
}
