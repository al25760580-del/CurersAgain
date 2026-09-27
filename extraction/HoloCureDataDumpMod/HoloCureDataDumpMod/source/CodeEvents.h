#pragma once
#include "ModuleMain.h"
#include <Aurie/shared.hpp>
#include <YYToolkit/YYTK_Shared.hpp>

using menuFunc = void (*)(void);

void PlayerManagerCreateAfter(std::tuple<CInstance*, CInstance*, CCode*, int, RValue*>& Args);
void FandomManagerCreateAfter(std::tuple<CInstance*, CInstance*, CCode*, int, RValue*>& Args);
void PlayerManagerStepBefore(std::tuple<CInstance*, CInstance*, CCode*, int, RValue*>& Args);
void InputManagerStepBefore(std::tuple<CInstance*, CInstance*, CCode*, int, RValue*>& Args);
void StageManagerCreateAfter(std::tuple<CInstance*, CInstance*, CCode*, int, RValue*>& Args);
void TitleScreenCreateBefore(std::tuple<CInstance*, CInstance*, CCode*, int, RValue*>& Args);