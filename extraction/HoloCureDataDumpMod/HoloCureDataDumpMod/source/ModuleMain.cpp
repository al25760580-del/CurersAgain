#pragma comment(lib, "d3d11.lib")

#include <Aurie/shared.hpp>
#include <YYToolkit/YYTK_Shared.hpp>
#include <CallbackManager/CallbackManagerInterface.h>
#include "ModuleMain.h"
#include "CodeEvents.h"
#include "ScriptFunctions.h"
#include "CommonFunctions.h"
#include <thread>
#include <d3d11.h>

using namespace Aurie;
using namespace YYTK;

RValue GMLVarIndexMapGMLHash[1001];

TRoutine origStructGetFromHashFunc;
TRoutine origStructSetFromHashFunc;

CallbackManagerInterface* callbackManagerInterfacePtr = nullptr;
YYTKInterface* g_ModuleInterface = nullptr;
YYRunnerInterface g_RunnerInterface;

CInstance* globalInstance = nullptr;
std::thread framePauseThread;

int objAttackControllerIndex = -1;

AurieStatus moduleInitStatus = AURIE_MODULE_INITIALIZATION_FAILED;

void initHooks()
{
	if (!AurieSuccess(callbackManagerInterfacePtr->RegisterBuiltinFunctionCallback(MODNAME, "struct_get_from_hash", nullptr, nullptr, &origStructGetFromHashFunc)))
	{
		DbgPrintEx(LOG_SEVERITY_ERROR, "Failed to register callback for %s", "struct_get_from_hash");
		return;
	}
	if (!AurieSuccess(callbackManagerInterfacePtr->RegisterBuiltinFunctionCallback(MODNAME, "struct_set_from_hash", nullptr, nullptr, &origStructSetFromHashFunc)))
	{
		DbgPrintEx(LOG_SEVERITY_ERROR, "Failed to register callback for %s", "struct_set_from_hash");
		return;
	}

	if (!AurieSuccess(callbackManagerInterfacePtr->RegisterScriptFunctionCallback(MODNAME, "gml_Script_AddTimeEvent@gml_Object_obj_StageManager_Other_10", AddTimeEventBefore, nullptr, nullptr)))
	{
		DbgPrintEx(LOG_SEVERITY_ERROR, "Failed to register callback for %s", "gml_Script_AddTimeEvent@gml_Object_obj_StageManager_Other_10");
		return;
	}



	if (!AurieSuccess(callbackManagerInterfacePtr->RegisterCodeEventCallback(MODNAME, "gml_Object_obj_PlayerManager_Create_0", nullptr, PlayerManagerCreateAfter)))
	{
		DbgPrintEx(LOG_SEVERITY_ERROR, "Failed to register callback for %s", "gml_Object_obj_PlayerManager_Create_0");
		return;
	}
	if (!AurieSuccess(callbackManagerInterfacePtr->RegisterCodeEventCallback(MODNAME, "gml_Object_obj_PlayerManager_Step_0", PlayerManagerStepBefore, nullptr)))
	{
		DbgPrintEx(LOG_SEVERITY_ERROR, "Failed to register callback for %s", "gml_Object_obj_PlayerManager_Step_0");
		return;
	}
	if (!AurieSuccess(callbackManagerInterfacePtr->RegisterCodeEventCallback(MODNAME, "gml_Object_obj_InputManager_Step_0", InputManagerStepBefore, nullptr)))
	{
		DbgPrintEx(LOG_SEVERITY_ERROR, "Failed to register callback for %s", "gml_Object_obj_InputManager_Step_0");
		return;
	}
	if (!AurieSuccess(callbackManagerInterfacePtr->RegisterCodeEventCallback(MODNAME, "gml_Object_obj_FandomManager_Create_0", nullptr, FandomManagerCreateAfter)))
	{
		DbgPrintEx(LOG_SEVERITY_ERROR, "Failed to register callback for %s", "gml_Object_obj_FandomManager_Create_0");
		return;
	}
	if (!AurieSuccess(callbackManagerInterfacePtr->RegisterCodeEventCallback(MODNAME, "gml_Object_obj_StageManager_Create_0", nullptr, StageManagerCreateAfter)))
	{
		DbgPrintEx(LOG_SEVERITY_ERROR, "Failed to register callback for %s", "gml_Object_obj_StageManager_Create_0");
		return;
	}
	if (!AurieSuccess(callbackManagerInterfacePtr->RegisterCodeEventCallback(MODNAME, "gml_Object_obj_TitleScreen_Create_0", TitleScreenCreateBefore, nullptr)))
	{
		DbgPrintEx(LOG_SEVERITY_ERROR, "Failed to register callback for %s", "gml_Object_obj_TitleScreen_Create_0");
		return;
	}


	g_RunnerInterface = g_ModuleInterface->GetRunnerInterface();
	g_ModuleInterface->GetGlobalInstance(&globalInstance);

	objAttackControllerIndex = static_cast<int>(g_ModuleInterface->CallBuiltin("asset_get_index", { "obj_AttackController" }).ToInt32());

	AurieStatus status = AURIE_SUCCESS;
	for (int i = 0; i < std::extent<decltype(VariableNamesStringsArr)>::value; i++)
	{
		if (!AurieSuccess(status))
		{
			DbgPrintEx(LOG_SEVERITY_ERROR, "Failed to get hash for %s", VariableNamesStringsArr[i]);
		}
		GMLVarIndexMapGMLHash[i] = std::move(g_ModuleInterface->CallBuiltin("variable_get_hash", { VariableNamesStringsArr[i] }));
	}

	moduleInitStatus = AURIE_SUCCESS;

	callbackManagerInterfacePtr->LogToFile(MODNAME, "Finished initializing");
}

void runnerInitCallback(FunctionWrapper<void(int)>& dummyWrapper)
{
	AurieStatus status = AURIE_SUCCESS;
	status = ObGetInterface("callbackManager", (AurieInterfaceBase*&)callbackManagerInterfacePtr);
	if (!AurieSuccess(status))
	{
		printf("Failed to get callback manager interface. Make sure that CallbackManagerMod is located in the mods/Aurie directory.\n");
		return;
	}

	callbackManagerInterfacePtr->RegisterInitFunction(initHooks);
}

EXPORTED AurieStatus ModulePreinitialize(
	IN AurieModule* Module,
	IN const fs::path& ModulePath
)
{
	AurieStatus status = AURIE_SUCCESS;
	// Gets a handle to the interface exposed by YYTK
	// You can keep this pointer for future use, as it will not change unless YYTK is unloaded.
	g_ModuleInterface = GetInterface();

	// If we can't get the interface, we fail loading.
	if (!AurieSuccess(status))
	{
		callbackManagerInterfacePtr->LogToFile(MODNAME, "Failed to get YYTK Interface");
		printf("Failed to get YYTK Interface\n");
		return AURIE_MODULE_DEPENDENCY_NOT_RESOLVED;
	}

	g_ModuleInterface->CreateCallback(
		Module,
		EVENT_RUNNER_INIT,
		runnerInitCallback,
		0
	);

	return AURIE_SUCCESS;
}

EXPORTED AurieStatus ModuleInitialize(
	IN AurieModule* Module,
	IN const fs::path& ModulePath
)
{
	return moduleInitStatus;
}