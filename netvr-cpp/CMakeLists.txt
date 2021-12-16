cmake_minimum_required(VERSION 3.17 FATAL_ERROR)
project(isbl_netvr LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)

add_library(${PROJECT_NAME} SHARED src/main.cpp)

if(WIN32)
    target_compile_definitions(${PROJECT_NAME} PRIVATE WIN_EXPORT)
endif()

if(WIN32)
    add_custom_command(TARGET ${PROJECT_NAME} POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:${PROJECT_NAME}>
        ${CMAKE_CURRENT_SOURCE_DIR}/../netvr-unity/Assets/Plugins/Windows/${CMAKE_GENERATOR_PLATFORM}/$<TARGET_FILE_NAME:${PROJECT_NAME}>
    )
elseif(ANDROID)
    add_custom_command(TARGET ${PROJECT_NAME} POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy $<TARGET_FILE:${PROJECT_NAME}>
        ${CMAKE_CURRENT_SOURCE_DIR}/../netvr-unity/Assets/Plugins/Android/${CMAKE_ANDROID_ARCH_ABI}/$<TARGET_FILE_NAME:${PROJECT_NAME}>
    )
endif()

message(STATUS ${CMAKE_CURRENT_BINARY_DIR})
message(STATUS ${ANDROID_ARCH})
message(STATUS ${CMAKE_ANDROID_ARCH_ABI})
