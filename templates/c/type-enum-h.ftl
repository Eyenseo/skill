#ifndef ${prefix_capital}TYPE_ENUM_H_
#define ${prefix_capital}TYPE_ENUM_H_

#include <stdbool.h>
#include <stdint.h>

typedef enum ${prefix}type_enum {
    ${prefix}CONSTANT_I8,
    ${prefix}CONSTANT_I16,
    ${prefix}CONSTANT_I32,
    ${prefix}CONSTANT_I64,
    ${prefix}CONSTANT_V64,
    ${prefix}I8,
    ${prefix}I16,
    ${prefix}I32,
    ${prefix}I64,
    ${prefix}V64,
    ${prefix}ANNOTATION,
    ${prefix}BOOLEAN,
    ${prefix}F32,
    ${prefix}F64,
    ${prefix}STRING,
    ${prefix}CONSTANT_LENGTH_ARRAY,
    ${prefix}VARIABLE_LENGTH_ARRAY,
    ${prefix}LIST,
    ${prefix}SET,
    ${prefix}MAP,
    ${prefix}USER_TYPE
} ${prefix}type_enum;

static inline const char *${prefix}type_enum_to_string ( ${prefix}type_enum type ) {
    static const char *strings[] = {
    "constant i8",
    "constant i16",
    "constant i32",
    "constant i64",
    "constant v64",
    "i8",
    "i16",
    "i32",
    "i64",
    "v64",
    "annotation",
    "boolean",
    "f32",
    "f64",
    "string",
    "constant length array",
    "variable length array",
    "list",
    "set",
    "map",
    "user defined type" };

    return strings[type];
}

int8_t ${prefix}type_enum_to_int ( ${prefix}type_enum type );

${prefix}type_enum ${prefix}type_enum_from_int ( int8_t type_id );

bool ${prefix}type_enum_is_constant ( ${prefix}type_enum type );

bool ${prefix}type_enum_is_container_type ( ${prefix}type_enum type );

#endif /* TYPE_ENUM_H_ */
