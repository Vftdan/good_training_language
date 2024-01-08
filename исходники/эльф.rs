use super::Результат;
use std::fs;
use std::path::Path;
use std::io::Write;
use компилятор::{ВидИнструкции, ПП};

#[derive(Debug)]
struct ЗаплаткаЦелейПрыжков {
    адрес_инструкции_прыжка: usize,
    адрес_операнда_прыжка: usize,
    индекс_инструкции_пп_цели: usize,
}

struct ЗаплаткаУказателяНаДанные {
    адрес_операнда_указателя: usize,
    смещение: usize,
}

fn сгенерировать_заплатку_целей_прыжков_32(код: &mut Vec<u8>, заплатки_целей_прыжков: &mut Vec<ЗаплаткаЦелейПрыжков>, индекс_инструкции_пп_цели: usize) {
    let адрес_операнда_прыжка = код.len();
    код.extend([0x00, 0x00, 0x00, 0x00]); // Заполняем операнд нулями, т.к. реальный относительный адрес будет известен позже.
    let адрес_инструкции_прыжка = код.len();
    заплатки_целей_прыжков.push(ЗаплаткаЦелейПрыжков {
        адрес_инструкции_прыжка,
        адрес_операнда_прыжка,
        индекс_инструкции_пп_цели,
    });
}

pub fn сгенерировать(путь_к_файлу: &Path, пп: &ПП, точка_входа_программы: usize) -> Результат<()> {
    let mut код = vec![];

    let mut адреса_инструкций_пп: Vec<usize> = Vec::new();
    let mut заплатки_целей_прыжков: Vec<ЗаплаткаЦелейПрыжков> = Vec::new();
    let mut заплатки_указателей_на_данные: Vec<ЗаплаткаУказателяНаДанные> = Vec::new();

    код.extend([0xE8]); // call
    сгенерировать_заплатку_целей_прыжков_32(&mut код, &mut заплатки_целей_прыжков, точка_входа_программы);
    код.extend([0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00]); // mov rax, SYS_exit (60)
    код.extend([0x48, 0xC7, 0xC7, 0x00, 0x00, 0x00, 0x00]); // mov rdi, 0
    код.extend([0x0F, 0x05]); // syscall

    for инструкция in &пп.код {
        адреса_инструкций_пп.push(код.len());
        match &инструкция.вид {
            ВидИнструкции::Ноп => {}
            // «Короткое» проталкивание (i8) "\x6A\x7F"
            // «Длинное» проталкивание (i32) "\x68\x00\x00\x00\x00"
            ВидИнструкции::ПротолкнутьЦелое(значение) => {
                assert!(*значение <= i32::MAX as usize);
                код.extend([0x68]); // push
                код.extend((*значение as i32).to_le_bytes());
                // СДЕЛАТЬ: реализовать поддержу «коротких» проталкиваний для целых чисел.
            }
            &ВидИнструкции::ПротолкнутьУказатель(смещение) => {
                код.extend([0x68]); // push
                let адрес_операнда_указателя = код.len();
                код.extend([0x00, 0x00, 0x00, 0x00]); // Заполняем операнд нулями, т.к. реальный указатель будет известен позже.
                заплатки_указателей_на_данные.push(ЗаплаткаУказателяНаДанные {
                    смещение,
                    адрес_операнда_указателя,
                });
            }
            &ВидИнструкции::Вытолкнуть(количество) => {
                // СДЕЛАТЬ: можеть быть стоит напрямую модифицировать регистр rsp одной операцией?
                for _ in 0..количество {
                    код.extend([0x58]); // pop rax
                }
            }
            ВидИнструкции::СохранитьКадр => {
                код.extend([0x55]);             // push rbp
                код.extend([0x48, 0x89, 0xE5]); // mov rbp, rsp
            }
            ВидИнструкции::ВосстановитьКадр => {
                код.extend([0x5D]);             // pop rbp
            }
            &ВидИнструкции::ПрочитатьКадр(смещение) => {
                код.extend([0x48, 0x8B, 0x85]);                   // mov rax,
                код.extend((-(смещение as i32 + 1)*8).to_le_bytes()); // [rbp-смещение-1]
                код.extend([0x50]);                               // push rax
            }
            &ВидИнструкции::ЗаписатьКадр(смещение) => {
                код.extend([0x58]); // pop rax
                код.extend([0x48, 0x89, 0x85]);
                код.extend((-(смещение as i32 + 1)*8).to_le_bytes()); // mov [rbp-смещение-1], rax
            }
            &ВидИнструкции::ВызватьПроцедуру(индекс_инструкции_пп_цели) => {
                код.extend([0xE8]); // call
                сгенерировать_заплатку_целей_прыжков_32(&mut код, &mut заплатки_целей_прыжков, индекс_инструкции_пп_цели);
            }
            ВидИнструкции::Записать64 => {
                код.extend([0x5E]);             // pop rsi
                код.extend([0x58]);             // pop rax
                код.extend([0x48, 0x89, 0x06]); // mov [rsi], rax
            }
            &ВидИнструкции::Записать8 => {
                код.extend([0x5E]);       // pop rsi
                код.extend([0x58]);       // pop rax
                код.extend([0x88, 0x06]); // mov [rsi], al
            }
            ВидИнструкции::Прочитать64 => {
                код.extend([0x5E]);             // pop rsi
                код.extend([0x48, 0x8B, 0x06]); // mov rax, [rsi]
                код.extend([0x50]);             // push rax
            }
            ВидИнструкции::ЦелМеньше => {
                код.extend([0x5B]);             // pop rbx
                код.extend([0x58]);             // pop rax
                код.extend([0x48, 0x31, 0xC9]); // xor rcx, rcx
                код.extend([0x48, 0x39, 0xD8]); // cmp rax, rbx
                код.extend([0x0F, 0x92, 0xC1]); // setb cl
                код.extend([0x51]);             // push rcx
                // СДЕЛАТЬ: можно ли использовать условное
                // перемещение для реализации инструкций сравнения?
            }
            ВидИнструкции::ЦелБольше => {
                код.extend([0x5B]);             // pop rbx
                код.extend([0x58]);             // pop rax
                код.extend([0x48, 0x31, 0xC9]); // xor rcx, rcx
                код.extend([0x48, 0x39, 0xD8]); // cmp rax, rbx
                код.extend([0x0F, 0x97, 0xC1]); // seta cl
                код.extend([0x51]);             // push rcx
            }
            ВидИнструкции::ЦелРавно => {
                код.extend([0x5B]);             // pop rbx
                код.extend([0x58]);             // pop rax
                код.extend([0x48, 0x31, 0xC9]); // xor rcx, rcx
                код.extend([0x48, 0x39, 0xD8]); // cmp rax, rbx
                код.extend([0x0F, 0x94, 0xC1]); // setz cl
                код.extend([0x51]);             // push rcx
            }
            ВидИнструкции::ЦелСложение => {
                код.extend([0x5B]);             // pop rbx
                код.extend([0x58]);             // pop rax
                код.extend([0x48, 0x01, 0xD8]); // add rax, rbx
                код.extend([0x50]);             // push rax
            }
            ВидИнструкции::ЦелВычитание => {
                код.extend([0x5B]);             // pop rbx
                код.extend([0x58]);             // pop rax
                код.extend([0x48, 0x29, 0xD8]); // sub rax, rbx
                код.extend([0x50]);             // push rax
            }
            ВидИнструкции::ЦелУмножение => {
                код.extend([0x5B]);             // pop rbx
                код.extend([0x58]);             // pop rax
                код.extend([0x48, 0x31, 0xD2]); // xor rdx, rdx
                код.extend([0x48, 0xF7, 0xE3]); // mul rbx
                код.extend([0x50]);             // push rax
            }
            ВидИнструкции::ЦелОстаток => {
                код.extend([0x5B]);             // pop rbx
                код.extend([0x58]);             // pop rax
                код.extend([0x48, 0x31, 0xD2]); // xor rdx, rdx
                код.extend([0x48, 0xF7, 0xF3]); // div rbx
                код.extend([0x52]);             // push rdx
            }
            ВидИнструкции::ЦелДеление => {
                код.extend([0x5B]);             // pop rbx
                код.extend([0x58]);             // pop rax
                код.extend([0x48, 0x31, 0xD2]); // xor rdx, rdx
                код.extend([0x48, 0xF7, 0xF3]); // div rbx
                код.extend([0x50]);             // push rax
            }
            ВидИнструкции::ЛогОтрицание => {
                код.extend([0x48, 0x31, 0xDB]); // xor rbx, rbx
                код.extend([0x58]);             // pop rax
                код.extend([0x48, 0x85, 0xC0]); // test rax, rax
                код.extend([0x0F, 0x94, 0xC3]); // setz bl
                код.extend([0x53]);             // push rbx
            }
            &ВидИнструкции::Прыжок(индекс_инструкции_пп_цели) => {
                код.extend([0xE9]); // jmp
                сгенерировать_заплатку_целей_прыжков_32(&mut код, &mut заплатки_целей_прыжков, индекс_инструкции_пп_цели);
            }
            &ВидИнструкции::УсловныйПрыжок(индекс_инструкции_пп_цели) => {
                код.extend([0x58]);             // pop rax
                код.extend([0x48, 0x85, 0xC0]); // test rax, rax
                код.extend([0x0F, 0x85]);       // jnz
                сгенерировать_заплатку_целей_прыжков_32(&mut код, &mut заплатки_целей_прыжков, индекс_инструкции_пп_цели);
            }
            ВидИнструкции::ПечатьСтроки => {
                код.extend([0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00]); // mov rax, 1 ; SYS_write
                код.extend([0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00]); // mov rdi, 1 ; stdout
                код.extend([0x5e]);                                     // pop rsi
                код.extend([0x5A]);                                     // pop rdx
                код.extend([0x0F, 0x05]);                               // syscall
            }
            ВидИнструкции::Ввод => {
                код.extend([0x48, 0xC7, 0xC0, 0x00, 0x00, 0x00, 0x00]); // mov rax, 0 ; SYS_read
                код.extend([0x48, 0xC7, 0xC7, 0x00, 0x00, 0x00, 0x00]); // mov rdi, 0 ; stdin
                код.extend([0x5A]);                                     // pop rdx
                код.extend([0x5e]);                                     // pop rsi
                код.extend([0x0F, 0x05]);                               // syscall
                код.extend([0x50]);                                     // push rax
            }
            ВидИнструкции::Возврат => {
                код.extend([0xC3]); // ret
            }
        }
    }

    for ЗаплаткаЦелейПрыжков {
        адрес_инструкции_прыжка,
        адрес_операнда_прыжка,
        индекс_инструкции_пп_цели
    } in заплатки_целей_прыжков {
        let операнд = &mut код[адрес_операнда_прыжка..адрес_операнда_прыжка+4];
        let адрес_инструкции_прыжка = адрес_инструкции_прыжка as i32;
        let адрес_инструкции_пп = адреса_инструкций_пп[индекс_инструкции_пп_цели] as i32;
        let относительный_адрес = адрес_инструкции_пп - адрес_инструкции_прыжка;
        операнд.copy_from_slice(&относительный_адрес.to_le_bytes());
    }

    let размер_заголовка_эльфа = 64;
    let размер_программного_заголовка = 56;
    let размер_заголовков: u64 = размер_заголовка_эльфа + размер_программного_заголовка;
    let точка_входа_эльфа: u64 = 0x400000;
    let начало_данных = точка_входа_эльфа + размер_заголовков + код.len() as u64;

    for ЗаплаткаУказателяНаДанные {
        адрес_операнда_указателя,
        смещение,
    } in заплатки_указателей_на_данные {
        let операнд = &mut код[адрес_операнда_указателя..адрес_операнда_указателя+4];
        let указатель_на_данные = начало_данных as i32 + смещение as i32;
        операнд.copy_from_slice(&указатель_на_данные.to_le_bytes());
    }

    let mut байты: Vec<u8> = Vec::new();
    байты.extend([0x7f, 0x45, 0x4c, 0x46,
                  0x02, 0x01, 0x01, 0x00,
                  0x00, 0x00, 0x00, 0x00,
                  0x00, 0x00, 0x00, 0x00]); // e_ident
    байты.extend(2u16.to_le_bytes()); // e_type
    байты.extend(62u16.to_le_bytes()); // e_machine
    байты.extend(1u32.to_le_bytes()); // e_version
    байты.extend((точка_входа_эльфа + размер_заголовков).to_le_bytes()); // e_entry
    байты.extend(64u64.to_le_bytes()); // e_phoff
    байты.extend(0u64.to_le_bytes()); // e_shoff
    байты.extend(0u32.to_le_bytes()); // e_flags
    байты.extend(64u16.to_le_bytes()); // e_ehsize
    байты.extend(56u16.to_le_bytes()); // e_phentsize
    байты.extend(1u16.to_le_bytes()); // e_phnum
    байты.extend(64u16.to_le_bytes()); // e_shentsize
    байты.extend(0u16.to_le_bytes()); // e_shnum
    байты.extend(0u16.to_le_bytes()); // e_shstrndx

    байты.extend(1u32.to_le_bytes()); // p_type
    байты.extend(7u32.to_le_bytes()); // p_flags
    байты.extend(0u64.to_le_bytes()); // p_offset
    байты.extend(точка_входа_эльфа.to_le_bytes()); // p_vaddr
    байты.extend(точка_входа_эльфа.to_le_bytes()); // p_paddr
    байты.extend((размер_заголовков + код.len() as u64 + пп.иниц_данные.len() as u64).to_le_bytes()); // p_filesz
    байты.extend((размер_заголовков + код.len() as u64 + пп.иниц_данные.len() as u64 + пп.размер_неиниц_данных as u64).to_le_bytes()); // p_memsz
    байты.extend(4096u64.to_le_bytes()); // p_align

    байты.extend(&код);
    байты.extend(&пп.иниц_данные);

    let mut файл = fs::File::create(путь_к_файлу).map_err(|ошибка| {
        eprintln!("ОШИБКА: не удалось открыть файл «{путь_к_файлу}»: {ошибка}",
                  путь_к_файлу = путь_к_файлу.display());
    })?;

    #[cfg(all(unix))] {
        use std::os::unix::fs::PermissionsExt;
        let mut права = файл.metadata().map_err(|ошибка| {
            eprintln!("ОШИБКА: не получилось прочитать метаданные файла «{путь_к_файлу}»: {ошибка}",
                      путь_к_файлу = путь_к_файлу.display());
        })?.permissions();
        права.set_mode(0o755);
        файл.set_permissions(права).map_err(|ошибка| {
            eprintln!("ОШИБКА: не получилось установить права для файла «{путь_к_файлу}»: {ошибка}",
                      путь_к_файлу = путь_к_файлу.display());
        })?;
    }

    match файл.write(&байты) {
        Ok(_) => {
            println!("ИНФО: сгенерирован файл «{путь_к_файлу}»",
                     путь_к_файлу = путь_к_файлу.display());
            Ok(())
        }
        Err(ошибка) => {
            eprintln!("ОШИБКА: не удалось записать файл «{путь_к_файлу}»: {ошибка}",
                      путь_к_файлу = путь_к_файлу.display());
            Err(())
        }
    }
}
