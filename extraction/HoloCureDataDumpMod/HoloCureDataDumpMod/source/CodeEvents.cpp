#include "CommonFunctions.h"
#include "CodeEvents.h"
#include "ModuleMain.h"
#include "imgui/imgui.h"
#include "imgui/imgui_impl_dx11.h"
#include "imgui/imgui_impl_win32.h"
#include <thread>

extern CallbackManagerInterface* callbackManagerInterfacePtr;
extern YYTKInterface* g_ModuleInterface;
extern CInstance* globalInstance;
extern YYRunnerInterface g_RunnerInterface;
extern int objAttackControllerIndex;

// Data
ID3D11Device* g_pd3dDevice = nullptr;
ID3D11DeviceContext* g_pd3dDeviceContext = nullptr;
IDXGISwapChain* g_pSwapChain = nullptr;
bool						g_SwapChainOccluded = false;
UINT						g_ResizeWidth = 0, g_ResizeHeight = 0;
ID3D11RenderTargetView* g_mainRenderTargetView = nullptr;

ImGuiIO io;

ImVec4 clear_color = ImVec4(0.45f, 0.55f, 0.60f, 1.00f);

void CreateRenderTarget();
void CleanupRenderTarget();

// Helper functions

bool CreateDeviceD3D(HWND hWnd)
{
	// Setup swap chain
	DXGI_SWAP_CHAIN_DESC sd;
	ZeroMemory(&sd, sizeof(sd));
	sd.BufferCount = 2;
	sd.BufferDesc.Width = 0;
	sd.BufferDesc.Height = 0;
	sd.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
	sd.BufferDesc.RefreshRate.Numerator = 60;
	sd.BufferDesc.RefreshRate.Denominator = 1;
	sd.Flags = DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH;
	sd.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
	sd.OutputWindow = hWnd;
	sd.SampleDesc.Count = 1;
	sd.SampleDesc.Quality = 0;
	sd.Windowed = TRUE;
	sd.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;

	UINT createDeviceFlags = 0;
	//createDeviceFlags |= D3D11_CREATE_DEVICE_DEBUG;
	D3D_FEATURE_LEVEL featureLevel;
	const D3D_FEATURE_LEVEL featureLevelArray[2] = { D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_10_0, };
	HRESULT res = D3D11CreateDeviceAndSwapChain(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, createDeviceFlags, featureLevelArray, 2, D3D11_SDK_VERSION, &sd, &g_pSwapChain, &g_pd3dDevice, &featureLevel, &g_pd3dDeviceContext);
	if (res == DXGI_ERROR_UNSUPPORTED) // Try high-performance WARP software driver if hardware is not available.
		res = D3D11CreateDeviceAndSwapChain(nullptr, D3D_DRIVER_TYPE_WARP, nullptr, createDeviceFlags, featureLevelArray, 2, D3D11_SDK_VERSION, &sd, &g_pSwapChain, &g_pd3dDevice, &featureLevel, &g_pd3dDeviceContext);
	if (res != S_OK)
		return false;

	CreateRenderTarget();
	return true;
}

void CleanupDeviceD3D()
{
	CleanupRenderTarget();
	if (g_pSwapChain) { g_pSwapChain->Release(); g_pSwapChain = nullptr; }
	if (g_pd3dDeviceContext) { g_pd3dDeviceContext->Release(); g_pd3dDeviceContext = nullptr; }
	if (g_pd3dDevice) { g_pd3dDevice->Release(); g_pd3dDevice = nullptr; }
}

void CreateRenderTarget()
{
	ID3D11Texture2D* pBackBuffer;
	g_pSwapChain->GetBuffer(0, IID_PPV_ARGS(&pBackBuffer));
	g_pd3dDevice->CreateRenderTargetView(pBackBuffer, nullptr, &g_mainRenderTargetView);
	pBackBuffer->Release();
}

void CleanupRenderTarget()
{
	if (g_mainRenderTargetView) { g_mainRenderTargetView->Release(); g_mainRenderTargetView = nullptr; }
}

// Forward declare message handler from imgui_impl_win32.cpp
extern IMGUI_IMPL_API LRESULT ImGui_ImplWin32_WndProcHandler(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam);

// Win32 message handler
// You can read the io.WantCaptureMouse, io.WantCaptureKeyboard flags to tell if dear imgui wants to use your inputs.
// - When io.WantCaptureMouse is true, do not dispatch mouse input data to your main application, or clear/overwrite your copy of the mouse data.
// - When io.WantCaptureKeyboard is true, do not dispatch keyboard input data to your main application, or clear/overwrite your copy of the keyboard data.
// Generally you may always pass all inputs to dear imgui, and hide them from your application based on those two flags.
LRESULT WINAPI WndProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam)
{
	if (ImGui_ImplWin32_WndProcHandler(hWnd, msg, wParam, lParam))
		return true;

	switch (msg)
	{
	case WM_SIZE:
		if (wParam == SIZE_MINIMIZED)
			return 0;
		g_ResizeWidth = (UINT)LOWORD(lParam); // Queue resize
		g_ResizeHeight = (UINT)HIWORD(lParam);
		return 0;
	case WM_SYSCOMMAND:
		if ((wParam & 0xfff0) == SC_KEYMENU) // Disable ALT application menu
			return 0;
		break;
	case WM_DESTROY:
		::PostQuitMessage(0);
		return 0;
	}
	return ::DefWindowProcW(hWnd, msg, wParam, lParam);
}

void PlayerManagerCreateAfter(std::tuple<CInstance*, CInstance*, CCode*, int, RValue*>& Args)
{
	CInstance* Self = std::get<0>(Args);
	{
		RValue MobsMap = g_ModuleInterface->CallBuiltin("variable_global_get", { "Mobs" });
		RValue mobsArr = g_ModuleInterface->CallBuiltin("ds_map_keys_to_array", { MobsMap });
		int mobsArrLength = static_cast<int>(g_ModuleInterface->CallBuiltin("array_length", { mobsArr }).m_Real);
		callbackManagerInterfacePtr->LogToFile(MODNAME, "mobMap:");
		for (int i = 0; i < mobsArrLength; i++)
		{
			RValue curMobKey = mobsArr[i];
			RValue curMob = g_ModuleInterface->CallBuiltin("ds_map_find_value", { MobsMap, curMobKey });
			callbackManagerInterfacePtr->LogToFile(MODNAME, "%s", curMob.ToString().data());
		}
	}

	{
		RValue FanLettersMap = g_ModuleInterface->CallBuiltin("variable_global_get", { "allFanLetters" });
		RValue fanLettersArr = g_ModuleInterface->CallBuiltin("ds_map_keys_to_array", { FanLettersMap });
		int fanLettersArrLength = static_cast<int>(g_ModuleInterface->CallBuiltin("array_length", { fanLettersArr }).m_Real);
		callbackManagerInterfacePtr->LogToFile(MODNAME, "fanLetters:");
		for (int i = 0; i < fanLettersArrLength; i++)
		{
			RValue key = fanLettersArr[i];
			RValue value = g_ModuleInterface->CallBuiltin("ds_map_find_value", { FanLettersMap, key });
			callbackManagerInterfacePtr->LogToFile(MODNAME, "%s", value.ToString().data());
		}
	}

	{
		RValue StickersMap = g_ModuleInterface->CallBuiltin("variable_instance_get", { Self, "STICKERS" });
		RValue stickersArr = g_ModuleInterface->CallBuiltin("ds_map_keys_to_array", { StickersMap });
		int stickersArrLength = static_cast<int>(g_ModuleInterface->CallBuiltin("array_length", { stickersArr }).m_Real);
		callbackManagerInterfacePtr->LogToFile(MODNAME, "stamps:");
		for (int i = 0; i < stickersArrLength; i++)
		{
			RValue key = stickersArr[i];
			RValue value = g_ModuleInterface->CallBuiltin("ds_map_find_value", { StickersMap, key });
			callbackManagerInterfacePtr->LogToFile(MODNAME, "%s", value.ToString().data());
		}
	}

	{
		RValue ItemsMap = g_ModuleInterface->CallBuiltin("variable_instance_get", { Self, "ITEMS" });
		RValue itemsArr = g_ModuleInterface->CallBuiltin("ds_map_keys_to_array", { ItemsMap });
		int itemsArrLength = static_cast<int>(g_ModuleInterface->CallBuiltin("array_length", { itemsArr }).m_Real);
		callbackManagerInterfacePtr->LogToFile(MODNAME, "items:");
		for (int i = 0; i < itemsArrLength; i++)
		{
			RValue key = itemsArr[i];
			RValue value = g_ModuleInterface->CallBuiltin("ds_map_find_value", { ItemsMap, key });
			callbackManagerInterfacePtr->LogToFile(MODNAME, "%s", value.ToString().data());
		}
	}

	{
		RValue characterDataMap = g_ModuleInterface->CallBuiltin("variable_global_get", { "characterData" });
		RValue characterDataArr = g_ModuleInterface->CallBuiltin("ds_map_keys_to_array", { characterDataMap });
		int characterDataArrLength = static_cast<int>(g_ModuleInterface->CallBuiltin("array_length", { characterDataArr }).m_Real);
		callbackManagerInterfacePtr->LogToFile(MODNAME, "characterData:");
		for (int i = 0; i < characterDataArrLength; i++)
		{
			RValue key = characterDataArr[i];
			RValue value = g_ModuleInterface->CallBuiltin("ds_map_find_value", { characterDataMap, key });
			callbackManagerInterfacePtr->LogToFile(MODNAME, "%s", value.ToString().data());
		}
	}

	{
		RValue attackMap = g_ModuleInterface->CallBuiltin("variable_instance_get", { objAttackControllerIndex, "attackIndex" });
		RValue attackArr = g_ModuleInterface->CallBuiltin("ds_map_keys_to_array", { attackMap });
		int attackArrLength = static_cast<int>(g_ModuleInterface->CallBuiltin("array_length", { attackArr }).m_Real);
		callbackManagerInterfacePtr->LogToFile(MODNAME, "attack:");
		for (int i = 0; i < attackArrLength; i++)
		{
			RValue key = attackArr[i];
			RValue value = g_ModuleInterface->CallBuiltin("ds_map_find_value", { attackMap, key });
			callbackManagerInterfacePtr->LogToFile(MODNAME, "%s", value.ToString().data());
		}
	}
}

void FandomManagerCreateAfter(std::tuple<CInstance*, CInstance*, CCode*, int, RValue*>& Args)
{
	CInstance* Self = std::get<0>(Args);
	RValue fanNames = g_ModuleInterface->CallBuiltin("variable_instance_get", { Self, "fanNames" });
	callbackManagerInterfacePtr->LogToFile(MODNAME, "%s", fanNames.ToString().data());
}

void StageManagerCreateAfter(std::tuple<CInstance*, CInstance*, CCode*, int, RValue*>& Args)
{
	RValue possibleDrops = g_ModuleInterface->CallBuiltin("variable_global_get", { "possibleDrops" });
	callbackManagerInterfacePtr->LogToFile(MODNAME, "%s", possibleDrops.ToString().data());
}

void PlayerManagerStepBefore(std::tuple<CInstance*, CInstance*, CCode*, int, RValue*>& Args)
{
	
}

void displayGMLVar(std::string& varName, RValue& varData)
{
	ImGui::Text("%s %s %s", varName.c_str(), varData.GetKindName().c_str(), varData.ToString().c_str());
}

void handleDisplayGMLVar(std::string varName, int index, RValue& varData)
{
	switch (varData.m_Kind)
	{
		case VALUE_ARRAY:
		{
			if (ImGui::TreeNode((void*)(intptr_t)index, "%s", varName.c_str()))
			{
				int arrLength = g_ModuleInterface->CallBuiltin("array_length", { varData }).ToInt32();
				for (int i = 0; i < arrLength; i++)
				{
					handleDisplayGMLVar(std::format("{}[{}]", varName, i), i, varData[i]);
				}
				ImGui::TreePop();
			}
			break;
		}
		case VALUE_REF:
		{
			RValue isMap = g_ModuleInterface->CallBuiltin("ds_exists", { varData, 1 });
			if (isMap.ToBoolean())
			{
				if (ImGui::TreeNode((void*)(intptr_t)index, "%s", varName.c_str()))
				{
					RValue varNamesArr = g_ModuleInterface->CallBuiltin("ds_map_keys_to_array", { varData });
					int varNamesArrLen = g_ModuleInterface->CallBuiltin("array_length", { varNamesArr }).ToInt32();
					for (int i = 0; i < varNamesArrLen; i++)
					{
						RValue nextVarName = varNamesArr[i];
						RValue nextVarData = g_ModuleInterface->CallBuiltin("ds_map_find_value", { varData, nextVarName });

						handleDisplayGMLVar(nextVarName.ToString(), i, nextVarData);
					}
					ImGui::TreePop();
				}
			}
			else
			{
				displayGMLVar(varName, varData);
			}
			break;
		}
		case VALUE_OBJECT:
		{
			RValue isStruct = g_ModuleInterface->CallBuiltin("is_struct", { varData });
			if (isStruct.ToBoolean())
			{
				RValue isMethod = g_ModuleInterface->CallBuiltin("is_method", { varData });
				if (!isMethod.ToBoolean())
				{
					if (ImGui::TreeNode((void*)(intptr_t)index, "%s", varName.c_str()))
					{
						RValue varNamesArr = g_ModuleInterface->CallBuiltin("variable_struct_get_names", { varData });
						int varNamesArrLen = g_ModuleInterface->CallBuiltin("array_length", { varNamesArr }).ToInt32();
						for (int i = 0; i < varNamesArrLen; i++)
						{
							RValue nextVarName = varNamesArr[i];
							RValue nextVarData = g_ModuleInterface->CallBuiltin("variable_instance_get", { varData, nextVarName });

							handleDisplayGMLVar(nextVarName.ToString(), i, nextVarData);
						}
						ImGui::TreePop();
					}
				}
			}
			else
			{
				displayGMLVar(varName, varData);
			}
			break;
		}
		case VALUE_STRING:
		case VALUE_BOOL:
		case VALUE_REAL:
		case VALUE_INT32:
		default:
		{
			displayGMLVar(varName, varData);
		}
	}
}

void handleInstanceDropdown(RValue& instance)
{
	RValue varNamesArr = g_ModuleInterface->CallBuiltin("variable_struct_get_names", { instance });
	int varNamesArrLen = g_ModuleInterface->CallBuiltin("array_length", { varNamesArr }).ToInt32();
	for (int i = 0; i < varNamesArrLen; i++)
	{
		RValue varName = varNamesArr[i];
		RValue varData = g_ModuleInterface->CallBuiltin("variable_instance_get", { instance, varName });
		
		handleDisplayGMLVar(varName.ToString(), i, varData);
	}
}

bool isGamePaused = false;

void handleImGUI()
{
	ImGui::Begin("Game Settings");
	if (ImGui::Button("Toggle Game Pause"))
	{
		isGamePaused = !isGamePaused;
	}
	ImGui::End();
	ImGui::Begin("Instance Variable Viewer");
	RValue instanceCountRValue;
	g_ModuleInterface->GetBuiltin("instance_count", nullptr, NULL_INDEX, instanceCountRValue);
	int instanceCount = instanceCountRValue.ToInt32();
	for (int i = 0; i < instanceCount; i++)
	{
		RValue instance;
		g_ModuleInterface->GetBuiltin("instance_id", nullptr, i, instance);
		RValue objectIndex = getInstanceVariable(instance, GML_object_index);
		RValue objectName = g_ModuleInterface->CallBuiltin("object_get_name", { objectIndex });
		if (ImGui::TreeNode((void*)(intptr_t)i, "%d %s", instance.ToInt32(), objectName.ToCString()))
		{
			handleInstanceDropdown(instance);
			ImGui::TreePop();
		}
	}
	ImGui::End();
	ImGui::Begin("Global Variable Viewer");
	if (ImGui::TreeNode((void*)(intptr_t)0, "Global Variables (WARNING: CURRENTLY VERY LAGGY)"))
	{
		RValue globalNamesArr = g_ModuleInterface->CallBuiltin("variable_struct_get_names", { globalInstance });
		int globalNamesArrLen = g_ModuleInterface->CallBuiltin("array_length", { globalNamesArr }).ToInt32();
		for (int i = 0; i < globalNamesArrLen; i++)
		{
			RValue varName = globalNamesArr[i];
			RValue varData = g_ModuleInterface->CallBuiltin("variable_instance_get", { globalInstance, varName });

			handleDisplayGMLVar(varName.ToString(), i, varData);
		}
		ImGui::TreePop();
	}
	ImGui::End();
}

void handleDrawGUI()
{
	if (g_pd3dDevice == nullptr)
	{
		return;
	}

	{
		// Poll and handle messages (inputs, window resize, etc.)
		// See the WndProc() function below for our to dispatch events to the Win32 backend.
		MSG msg;
		while (::PeekMessage(&msg, nullptr, 0U, 0U, PM_REMOVE))
		{
			::TranslateMessage(&msg);
			::DispatchMessage(&msg);
			if (msg.message == WM_QUIT)
			{
				isGamePaused = false;
			}
		}

		// Handle window being minimized or screen locked
		if (g_SwapChainOccluded && g_pSwapChain->Present(0, DXGI_PRESENT_TEST) == DXGI_STATUS_OCCLUDED)
		{
			::Sleep(10);
			return;
		}
		g_SwapChainOccluded = false;

		// Handle window resize (we don't resize directly in the WM_SIZE handler)
		if (g_ResizeWidth != 0 && g_ResizeHeight != 0)
		{
			CleanupRenderTarget();
			g_pSwapChain->ResizeBuffers(0, g_ResizeWidth, g_ResizeHeight, DXGI_FORMAT_UNKNOWN, 0);
			g_ResizeWidth = g_ResizeHeight = 0;
			CreateRenderTarget();
		}

		// Start the Dear ImGui frame
		ImGui_ImplDX11_NewFrame();
		ImGui_ImplWin32_NewFrame();
		ImGui::NewFrame();

		handleImGUI();

		// Rendering
		ImGui::Render();
		const float clear_color_with_alpha[4] = { clear_color.x * clear_color.w, clear_color.y * clear_color.w, clear_color.z * clear_color.w, clear_color.w };
		g_pd3dDeviceContext->OMSetRenderTargets(1, &g_mainRenderTargetView, nullptr);
		g_pd3dDeviceContext->ClearRenderTargetView(g_mainRenderTargetView, clear_color_with_alpha);
		ImGui_ImplDX11_RenderDrawData(ImGui::GetDrawData());

		// Present
		HRESULT hr = g_pSwapChain->Present(1, 0);   // Present with vsync
		//HRESULT hr = g_pSwapChain->Present(0, 0); // Present without vsync
		g_SwapChainOccluded = (hr == DXGI_STATUS_OCCLUDED);
	}
}

void InputManagerStepBefore(std::tuple<CInstance*, CInstance*, CCode*, int, RValue*>& Args)
{
	do
	{
		if (isGamePaused)
		{
			std::this_thread::sleep_for(std::chrono::milliseconds(10));
		}
		handleDrawGUI();
	}
	while (isGamePaused);
}

void initGUIWindow()
{
	if (g_pd3dDevice == nullptr)
	{
		// Create application window
		//ImGui_ImplWin32_EnableDpiAwareness();
		WNDCLASSEXW wc = { sizeof(wc), CS_CLASSDC, WndProc, 0L, 0L, GetModuleHandle(nullptr), nullptr, nullptr, nullptr, nullptr, L"Data Dump Mod Menu", nullptr };
		::RegisterClassExW(&wc);
		HWND hwnd = ::CreateWindowW(wc.lpszClassName, L"Data Dump Mod Menu", WS_OVERLAPPEDWINDOW, 100, 100, 1280, 720, nullptr, nullptr, wc.hInstance, nullptr);
		// Initialize Direct3D
		if (!CreateDeviceD3D(hwnd))
		{
			CleanupDeviceD3D();
			::UnregisterClassW(wc.lpszClassName, wc.hInstance);
			return;
		}

		// Show the window
		::ShowWindow(hwnd, SW_SHOWDEFAULT);
		::UpdateWindow(hwnd);

		IMGUI_CHECKVERSION();
		ImGui::CreateContext();
		io = ImGui::GetIO();
		io.ConfigFlags |= ImGuiConfigFlags_NavEnableKeyboard;     // Enable Keyboard Controls
		io.ConfigFlags |= ImGuiConfigFlags_NavEnableGamepad;      // Enable Gamepad Controls

		ImGui::StyleColorsDark();
		// Setup Platform/Renderer backends
		ImGui_ImplWin32_Init(hwnd);
		ImGui_ImplDX11_Init(g_pd3dDevice, g_pd3dDeviceContext);
	}
}

void TitleScreenCreateBefore(std::tuple<CInstance*, CInstance*, CCode*, int, RValue*>& Args)
{
	initGUIWindow();
}