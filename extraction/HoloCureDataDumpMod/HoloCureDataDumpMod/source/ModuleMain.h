#pragma once
#include <Aurie/shared.hpp>
#include <YYToolkit/YYTK_Shared.hpp>
#include <CallbackManager/CallbackManagerInterface.h>

#define VERSION_NUM "v1.1.1"
#define MODNAME "Holocure Data Dump Mod " VERSION_NUM

#define SOME_ENUM(DO) \
	DO(timePause) \
	DO(optionIcon) \
	DO(object_index) \

#define MAKE_ENUM(VAR) GML_ ## VAR,
enum VariableNames
{
	SOME_ENUM(MAKE_ENUM)
};

#define MAKE_STRINGS(VAR) #VAR,
const char* const VariableNamesStringsArr[] =
{
	SOME_ENUM(MAKE_STRINGS)
};

extern RValue GMLVarIndexMapGMLHash[1001];
extern CInstance* globalInstance;
extern YYTKInterface* g_ModuleInterface;
extern YYRunnerInterface g_RunnerInterface;

extern TRoutine origStructGetFromHashFunc;
extern TRoutine origStructSetFromHashFunc;